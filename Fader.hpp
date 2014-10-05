#ifndef FADER_HPP
#define FADER_HPP

#include <string>
#include <list>

#include <jack/jack.h>

class FaderSource
{
public:
  virtual void mix(jack_nframes_t nframes, jack_default_audio_sample_t *dest_buf, float gain) =0;
  virtual bool isDone() const = 0;
  virtual ~FaderSource() {}
};

class Fader : public FaderSource
{
protected:
  std::string name;
  std::list<FaderSource*> sources;
  float fader_gain;
  Fader *downstream;
  bool mixed;

  // Jack
  jack_port_t *out_port;

public:
  Fader(const std::string &name) : name(name),  fader_gain(1.0), downstream(nullptr),
				   mixed(false), out_port(nullptr) {}
  virtual ~Fader() {}
  void mix(jack_nframes_t nframes, jack_default_audio_sample_t *dest_buf, float gain);

  void addSource(FaderSource *src) {sources.push_back(src);}
  void unmarkMixed() {mixed = false;}
  void setDownstream(Fader *fader) {downstream = fader;}
  void setGain(float g) {fader_gain = g;}
  const std::string &getName() {return name;}
  void registerJackPorts(jack_client_t *client);
  bool isDone() const {return false;}
};

#endif // FADER_HPP
