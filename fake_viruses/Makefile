# Build the benign detector test specimens.
#
# Each specimen exhibits one suspicious-looking but harmless behavior so the
# Bio-Digital Defense engine has something live to observe and quarantine.
#
#   make          - build all specimens
#   make disease  - build the network-beacon specimen
#   make virus    - build the bulk-file-reader specimen
#   make bacteria - build the process-replicator specimen
#   make clean    - remove built binaries

CXX      ?= c++
CXXFLAGS ?= -std=c++17 -O2 -Wall -Wextra -pthread

SPECIMENS := disease virus bacteria

.PHONY: all clean
all: $(SPECIMENS)

# Each specimen is a single translation unit named after the specimen.
$(SPECIMENS): %: %.cpp
	$(CXX) $(CXXFLAGS) -o $@ $<

clean:
	rm -f $(SPECIMENS)
