#include "Sample.hpp"

inline size_t Sample::size() const
{
  return sample_length;
}

inline jack_default_audio_sample_t Sample::getFrame(jack_nframes_t frame) const
{
  //std::cout << "Getting frame from sample data" << sample_data << std::endl;
  return sample_data[frame];
}

const Sample *RoundRobinSample::getNextSample()
{
  const Sample *s = (*this)[current_sample];
  current_sample = (current_sample + 1) % size();
  return s;
}
