%module tdrum

%include "std_string.i"

%{
#include "TDrum.hpp"
%}

class Instrument
{
 public:
  bool loadSample(const std::string &path);
};

class Core
{
 public:
  void addInstrument(unsigned short key, Instrument* instr);
};
