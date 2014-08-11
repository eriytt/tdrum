#ifndef SAMPLE_HPP
#define SAMPLE_HPP

#include <vector>

#include <jack/jack.h>

class Sample
{
protected:
  unsigned int refcount;
  jack_default_audio_sample_t *sample_data;
  size_t sample_length;

  void incRef();
  void decRef();

public:
  Sample(jack_default_audio_sample_t *data, size_t size) : refcount(0), sample_data(data), sample_length(size) {}
  size_t size() const;
  jack_default_audio_sample_t getFrame(jack_nframes_t frame) const;
};

class RoundRobinSample : public std::vector<const Sample *>
{
protected:
  unsigned int current_sample;

public:
  RoundRobinSample() : current_sample(0) {}
  const Sample *getNextSample();
};

#endif // SAMPLE_HPP
