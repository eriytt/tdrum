#ifndef TDRUM_HPP
#define TDRUM_HPP
#include <map>
#include <vector>

#include <jack/jack.h>
#include <jack/midiport.h>


class Sample
{
protected:
  unsigned int refcount;
  jack_default_audio_sample_t *sample_data;
  size_t sample_length;

  void incRef();
  void decRef();

public:
  Sample(jack_default_audio_sample_t *data, size_t size) : sample_data(data), sample_length(size) {}
  size_t size() const;
  jack_default_audio_sample_t getFrame(jack_nframes_t frame) const;
};

class RoundRobinSample
{
protected:
  unsigned int current_sample;
  std::vector<Sample> samples;
};

class SampleRegistry
{
protected:
  std::map<Sample *, Sample *> samples;
};


class Instrument
{
protected:
  std::vector<const Sample *> samples;

protected:
  void addSample(const Sample *sample);

public:
  bool loadSample(const std::string &path);
  const Sample *getSampleForVelocity(unsigned char velocity);
};

class PlayingSample
{
protected:
  unsigned int current_position;
  const Sample *sample;

public:
  PlayingSample(const Sample *sample) : current_position(0), sample(sample) {}
  jack_default_audio_sample_t getNextFrame();
  bool isDone() const;
};

class Core
{
protected:
  std::map<unsigned short, Instrument*> keyToInstrument;
  // TODO: vector is a bad choice here because erase is very inefficient.
  std::vector<PlayingSample> playing_samples;

  SampleRegistry registry;


  // Jack stuff
protected:
  jack_client_t *jack_client;
  jack_port_t *midi_input_port;
  jack_port_t *audio_output_port;
  static int JackProcessTrampoline(jack_nframes_t nframes, void *arg);
  int jackProcess(jack_nframes_t nframes);

protected:
  // Sound engine stuff
  void mixInstrument(unsigned short key, unsigned char velocity);
  void mix(jack_nframes_t nframes, jack_default_audio_sample_t *dest_buf);

public:
  Core(): jack_client(nullptr), midi_input_port(nullptr), audio_output_port(nullptr) {}
  void addInstrument(unsigned short key, Instrument* instr);
  bool registerJack();
};

class Notify
{
public:
  typedef enum class
  {
    ERROR = 1,
    WARNING,
    INFO,
    VERBOSE,
    TRACE,
    DEBUG
  } NotifierType;

public:
  static void notify(NotifierType t, const std::string &what, const std::string &why);

private:
  static const Notify &singleton;
  Notify();
};


#endif // TDRUM_HPP
