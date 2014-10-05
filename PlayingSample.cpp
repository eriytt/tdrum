#include "PlayingSample.hpp"

#include <iostream>

#include "Sample.hpp"

PlayingSample::PlayingSample(const Sample *sample, unsigned int offset, Fader *fader) : current_position(-offset), sample(sample), fader(fader)
{
  //std::cout << "New PlayingSample " << this << ", Sample : " << sample << std::endl;
}

inline jack_default_audio_sample_t PlayingSample::getNextFrame()
{
  if (isDone() or ++current_position < 0)
    return 0.0;
  return sample->getFrame(current_position);
}

inline bool PlayingSample::isDone() const
{
  return current_position >= static_cast<int>(sample->size());
}

void PlayingSample::mix(jack_nframes_t nframes, jack_default_audio_sample_t *dest_buf, float gain)
{
  // std::cout << "Mixing playing sample @" << gain << std::endl;
  for (unsigned int i = 0; i < nframes and (not isDone()); i++)
    dest_buf[i] += gain * getNextFrame();
}
