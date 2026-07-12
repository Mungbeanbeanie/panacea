// bacteria.cpp — benign test specimen: "process replicator"
//
// Purpose
//   A harmless demo program for the Bio-Digital Defense detector. It exhibits a
//   single suspicious-looking behavior — rapidly spawning short-lived child
//   processes — so the endpoint engine has a live "bacterium" to observe, classify,
//   and quarantine. This mimics the process-table churn of self-replicating malware,
//   made safe by a hard cap on how many children exist at once.
//
//   Biological framing: bacteria reproduce by dividing. Here that maps to a bounded
//   burst of fork()/exit() each tick — the process-spawn signature detectors watch
//   for — deliberately NOT an unbounded fork bomb.
//
// Behavior
//   Once per second, fork a fixed, small number of child processes. Each child
//   lingers briefly (kChildLinger) so detectors polling the process table can
//   observe it; the parent reaps them all before the next tick.
//
// Usage
//   ./bacteria [children_per_tick]
//   Default: 5. Values are clamped to a safe maximum.
//
// Safety
//   Bounded replication. At most kMaxChildrenPerTick children per second, each
//   reaped before the next tick, so total live processes stay tiny. Stops cleanly
//   on Ctrl+C (SIGINT/SIGTERM).

#include <sys/wait.h>
#include <unistd.h>

#include <atomic>
#include <chrono>
#include <csignal>
#include <cstdlib>
#include <iostream>
#include <thread>
#include <vector>

namespace {

// Flipped by the signal handler to request a graceful shutdown.
std::atomic<bool> g_running{true};

// How long to wait between replication bursts.
constexpr auto kTickInterval = std::chrono::seconds(1);

// Default and hard-capped number of children spawned per tick. The cap is what
// keeps this a safe demo rather than a fork bomb.
constexpr int kDefaultChildrenPerTick = 5;
constexpr int kMaxChildrenPerTick = 20;

// How long each child lingers before exiting, so an observer polling the process
// table (~500ms cadence) can actually see the children. Instant _exit(0) children
// live sub-millisecond lives and are invisible to any realistic poll — same reason
// virus.cpp holds its file descriptors open (kHoldOpenDuration).
constexpr auto kChildLinger = std::chrono::milliseconds(700);

// Handle SIGINT/SIGTERM by asking the main loop to stop.
void RequestStop(int /*signal*/) { g_running = false; }

// Fork `count` children that each exit immediately, then reap them all.
// Returns the number of children successfully spawned.
int ReplicateOnce(int count) {
  std::vector<pid_t> children;
  children.reserve(count);

  for (int i = 0; i < count; ++i) {
    pid_t pid = fork();
    if (pid == 0) {
      // Child: linger long enough to be observable, then exit.
      std::this_thread::sleep_for(kChildLinger);
      _exit(0);
    }
    if (pid > 0) {
      children.push_back(pid);
    }
    // pid < 0 means fork failed; skip and keep the burst bounded.
  }

  for (pid_t pid : children) {
    waitpid(pid, nullptr, 0);
  }
  return static_cast<int>(children.size());
}

// Parse and clamp the requested children-per-tick from argv.
int ParseChildrenPerTick(int argc, char** argv) {
  if (argc <= 1) {
    return kDefaultChildrenPerTick;
  }
  int requested = std::atoi(argv[1]);
  if (requested < 1) {
    requested = 1;
  }
  if (requested > kMaxChildrenPerTick) {
    requested = kMaxChildrenPerTick;
  }
  return requested;
}

}  // namespace

int main(int argc, char** argv) {
  const int children_per_tick = ParseChildrenPerTick(argc, argv);

  std::signal(SIGINT, RequestStop);
  std::signal(SIGTERM, RequestStop);

  std::cout << "[bacteria] process replicator -> " << children_per_tick
            << " children/tick (Ctrl+C to stop)\n";

  unsigned long tick = 0;
  while (g_running) {
    const int spawned = ReplicateOnce(children_per_tick);
    std::cout << "[bacteria] tick #" << ++tick << " spawned " << spawned
              << " children" << std::endl;
    std::this_thread::sleep_for(kTickInterval);
  }

  std::cout << "[bacteria] stopped after " << tick << " ticks\n";
  return 0;
}
