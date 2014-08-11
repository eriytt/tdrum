#ifndef FADER_HPP
#define FADER_HPP

#include <list>

#include <jack/jack.h>

class FaderSource
{
public:
  virtual void mix(jack_nframes_t nframes, jack_default_audio_sample_t *dest_buf, float gain) =0;
};

class Fader : public FaderSource
{
protected:
  std::list<FaderSource*> sources;
  jack_port_t *out_port;
  float fader_gain;
  Fader *downstream;
  bool mixed;

public:
  Fader() : out_port(nullptr), fader_gain(0), downstream(nullptr), mixed(false) {}
  void mix(jack_nframes_t nframes, jack_default_audio_sample_t *dest_buf, float gain);

  void addSource(FaderSource *src) {sources.push_back(src);}
  void unmarkMixed() {mixed = false;}
  void setDownstream(Fader *fader) {downstream = fader;}
  void setGain(float g) {fader_gain = g;}
};

#endif // FADER_HPP
