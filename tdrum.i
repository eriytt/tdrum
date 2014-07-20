%module tdrum

%include "std_string.i"

%{
#include "TDrum.hpp"
%}

class Instrument
{
 public:
  bool loadSample(const std::string &path, unsigned char velocity);
};

class Core
{
 public:
%apply SWIGTYPE *DISOWN {Instrument* instr};
  void addInstrument(unsigned short key, Instrument* instr);
  bool registerJack();
};
