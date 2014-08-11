%module tdrum

%include "std_string.i"

%{
#include "TDrum.hpp"
#include "Instrument.hpp"
#include "PlayingSample.hpp"
%}

class Instrument
{
 public:
  bool loadSample(const std::string &path, unsigned char velocity);
};

class Fader
{
 public:
  void addSource(FaderSource *src);
  void unmarkMixed();
%apply SWIGTYPE *DISOWN {Fader *fader};
  void setDownstream(Fader *fader);
  void setGain(float g);
};

class Core
{
 public:
%apply SWIGTYPE *DISOWN {Instrument* instr};
  void addInstrument(unsigned short key, Instrument* instr);
  bool registerJack();
};
