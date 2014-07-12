#include "TDrum.hpp"

#include <iostream>

bool Instrument::loadSample(const std::string &path)
{
  std::cout << "Loading sample " << path << std::endl;
  return true;
}

void Core::addInstrument(unsigned short key, Instrument* instr)
{
  std::cout << "Adding instrument as key " << key << std::endl;
}
