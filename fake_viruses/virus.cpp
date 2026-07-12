// virus.cpp — benign test specimen: "bulk file reader"
//
// Purpose
//   A harmless demo program for the Bio-Digital Defense detector. It exhibits a
//   single suspicious-looking behavior — rapidly opening and reading many files
//   in a burst — so the endpoint engine has a live "virus" to observe, classify,
//   and quarantine. This is the access pattern of ransomware-style scanners, made
//   safe: it only reads, never writes, modifies, or deletes.
//
//   Biological framing: a "virus" invades many cells at once. Here that maps to a
//   fan-out of concurrent file reads, the file-access signature detectors watch for.
//
// Behavior
//   Once per second, enumerate files in a target directory and read the first
//   chunk of each, opening several at a time via worker threads.
//
// Usage
//   ./virus [directory]
//   Default: the current working directory.
//
// Safety
//   Read-only. Files are opened O_RDONLY, a small prefix is read and discarded,
//   then closed. Bounded to one sweep per second. Stops cleanly on Ctrl+C.

#include <dirent.h>
#include <fcntl.h>
#include <unistd.h>

#include <atomic>
#include <chrono>
#include <csignal>
#include <iostream>
#include <string>
#include <thread>
#include <vector>

namespace {

// Flipped by the signal handler to request a graceful shutdown.
std::atomic<bool> g_running{true};

// How long to wait between file-read sweeps.
constexpr auto kSweepInterval = std::chrono::seconds(1);

// Bytes to read from the front of each file (then discarded).
constexpr size_t kReadChunkBytes = 4096;

// How long to hold each opened file descriptor before closing it — long enough for a
// polling-based detector (e.g. a periodic `lsof` snapshot) to actually observe the
// concurrent-open burst. Since all readers in a batch are joined together, holding each
// one individually keeps the whole batch open concurrently during the shared window.
constexpr auto kHoldOpenDuration = std::chrono::milliseconds(200);

// How many files to open concurrently per sweep.
constexpr size_t kConcurrentReaders = 8;

// Handle SIGINT/SIGTERM by asking the main loop to stop.
void RequestStop(int /*signal*/) { g_running = false; }

// Return the paths of the regular files directly inside `directory`.
std::vector<std::string> ListFiles(const std::string& directory) {
  std::vector<std::string> paths;
  DIR* dir = opendir(directory.c_str());
  if (dir == nullptr) {
    return paths;
  }
  for (dirent* entry = readdir(dir); entry != nullptr; entry = readdir(dir)) {
    const std::string name = entry->d_name;
    if (name == "." || name == "..") {
      continue;
    }
    paths.push_back(directory + "/" + name);
  }
  closedir(dir);
  return paths;
}

// Open one file read-only, read a small prefix, discard it, and close.
// Returns true if the file was opened successfully.
bool ReadFilePrefix(const std::string& path) {
  int fd = open(path.c_str(), O_RDONLY);
  if (fd < 0) {
    return false;
  }
  std::vector<char> buffer(kReadChunkBytes);
  ssize_t bytes_read = read(fd, buffer.data(), buffer.size());
  (void)bytes_read;  // content intentionally read and discarded
  std::this_thread::sleep_for(kHoldOpenDuration);
  close(fd);
  return true;
}

// Read every path, launching up to kConcurrentReaders threads at a time so the
// reads visibly fan out. Returns the number of files opened successfully.
size_t ReadFilesConcurrently(const std::vector<std::string>& paths) {
  std::atomic<size_t> opened{0};
  for (size_t start = 0; start < paths.size(); start += kConcurrentReaders) {
    std::vector<std::thread> workers;
    const size_t end = std::min(start + kConcurrentReaders, paths.size());
    for (size_t i = start; i < end; ++i) {
      workers.emplace_back([&opened, &paths, i]() {
        if (ReadFilePrefix(paths[i])) {
          ++opened;
        }
      });
    }
    for (std::thread& worker : workers) {
      worker.join();
    }
  }
  return opened;
}

}  // namespace

int main(int argc, char** argv) {
  const std::string directory = (argc > 1) ? argv[1] : ".";

  std::signal(SIGINT, RequestStop);
  std::signal(SIGTERM, RequestStop);

  std::cout << "[virus] bulk file reader -> " << directory
            << " (Ctrl+C to stop)\n";

  unsigned long tick = 0;
  while (g_running) {
    const std::vector<std::string> files = ListFiles(directory);
    const size_t opened = ReadFilesConcurrently(files);
    std::cout << "[virus] sweep #" << ++tick << " read " << opened << "/"
              << files.size() << " files" << std::endl;
    std::this_thread::sleep_for(kSweepInterval);
  }

  std::cout << "[virus] stopped after " << tick << " sweeps\n";
  return 0;
}
