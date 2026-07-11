// disease.cpp — benign test specimen: "network beacon"
//
// Purpose
//   A harmless demo program for the Bio-Digital Defense detector. It exhibits a
//   single suspicious-looking behavior — repeated outbound network contact — so
//   the endpoint engine has a live "disease" to observe, classify, and quarantine.
//   It carries no payload: it opens a TCP socket, immediately closes it, and loops.
//
//   Biological framing: a "disease" spreads through the network. Here that maps to
//   a periodic outbound beacon, the network-activity signature detectors watch for.
//
// Behavior
//   Once per second, open a TCP connection to a target host:port, then close it.
//
// Usage
//   ./disease [host] [port]
//   Defaults: host=8.8.8.8  port=53   (a public DNS endpoint; connect-and-close only)
//
// Safety
//   Read-only network probe. No data is sent or received. Bounded to one attempt
//   per second. Stops cleanly on Ctrl+C (SIGINT/SIGTERM).

#include <arpa/inet.h>
#include <netdb.h>
#include <sys/socket.h>
#include <unistd.h>

#include <atomic>
#include <chrono>
#include <csignal>
#include <cstdlib>
#include <cstring>
#include <iostream>
#include <string>
#include <thread>

namespace {

// Flipped by the signal handler to request a graceful shutdown.
std::atomic<bool> g_running{true};

// Default beacon target: a public DNS endpoint we only connect to and drop.
constexpr const char* kDefaultHost = "8.8.8.8";
constexpr const char* kDefaultPort = "53";

// How long to wait between beacon attempts.
constexpr auto kBeaconInterval = std::chrono::seconds(1);

// Handle SIGINT/SIGTERM by asking the main loop to stop.
void RequestStop(int /*signal*/) { g_running = false; }

// Open a TCP connection to host:port and immediately close it.
// Returns true if the connection succeeded. Sends and receives no data.
bool BeaconOnce(const std::string& host, const std::string& port) {
  addrinfo hints{};
  hints.ai_family = AF_UNSPEC;      // IPv4 or IPv6
  hints.ai_socktype = SOCK_STREAM;  // TCP

  addrinfo* results = nullptr;
  if (getaddrinfo(host.c_str(), port.c_str(), &hints, &results) != 0) {
    return false;
  }

  bool connected = false;
  for (addrinfo* addr = results; addr != nullptr; addr = addr->ai_next) {
    int fd = socket(addr->ai_family, addr->ai_socktype, addr->ai_protocol);
    if (fd < 0) {
      continue;
    }
    if (connect(fd, addr->ai_addr, addr->ai_addrlen) == 0) {
      connected = true;
    }
    close(fd);
    if (connected) {
      break;
    }
  }

  freeaddrinfo(results);
  return connected;
}

}  // namespace

int main(int argc, char** argv) {
  const std::string host = (argc > 1) ? argv[1] : kDefaultHost;
  const std::string port = (argc > 2) ? argv[2] : kDefaultPort;

  std::signal(SIGINT, RequestStop);
  std::signal(SIGTERM, RequestStop);

  std::cout << "[disease] network beacon -> " << host << ":" << port
            << " (Ctrl+C to stop)\n";

  unsigned long tick = 0;
  while (g_running) {
    const bool ok = BeaconOnce(host, port);
    std::cout << "[disease] beacon #" << ++tick << " "
              << (ok ? "connected" : "failed") << std::endl;
    std::this_thread::sleep_for(kBeaconInterval);
  }

  std::cout << "[disease] stopped after " << tick << " beacons\n";
  return 0;
}
