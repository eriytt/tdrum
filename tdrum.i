%module tdrum

%include "std_string.i"

%{
#include "TDrum.hpp"
#include "Instrument.hpp"
#include "PlayingSample.hpp"
%}

%nodefault FaderSource;  // FaderSource is abstract thus has no constructor
class FaderSource
{
};

class Fader : public FaderSource
{
 public:
  Fader(const std::string &name);
  void addSource(FaderSource *src);
%apply SWIGTYPE *DISOWN {Fader *fader};
  void setDownstream(Fader *fader);
  void setGain(float g);
  void registerJackPorts(jack_client_t *client);
};

class Instrument
{
 public:
  bool loadSample(const std::string &path, unsigned char velocity);
  void setFader(Fader *f);
};

class Core
{
 public:
%apply SWIGTYPE *DISOWN {Instrument* instr};
  void addInstrument(unsigned short key, Instrument* instr);
%apply SWIGTYPE *DISOWN {Fader* fader};
  void addFader(Fader *fader);
  jack_client_t *registerJack();
};
