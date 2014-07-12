#include <map>
#include <vector>

class Sample
{
protected:
  unsigned int refcount;
  void *sample_data; // TODO: make this the jack sample type

  void incRef();
  void decRef();
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

public:
  bool loadSample(const std::string &path);
};


class Instrument
{
protected:
  std::vector<const Sample *> samples;

public:
  bool loadSample(const std::string &path);
};

class PlayingSample
{
  unsigned int current_position;
  const Sample &sample;

  PlayingSample(const Sample &sample) : current_position(0), sample(sample) {}
};

class Core
{
protected:
  std::map<unsigned short, Instrument*> keyToInstrument;
  std::vector<PlayingSample> playing_samples;

  SampleRegistry registry;

public:
  void addInstrument(unsigned short key, Instrument* instr);
  void registerJack();
};
