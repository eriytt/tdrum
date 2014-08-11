#ifndef PLAYINGSAMPLE_HPP
#define PLAYINGSAMPLE_HPP

#include "Fader.hpp"

class Sample;

class PlayingSample : public FaderSource
{
protected:
  int current_position;
  const Sample *sample;
  Fader *fader;

public:
  PlayingSample(const Sample *sample, unsigned int offset, Fader *);
  jack_default_audio_sample_t getNextFrame();
  bool isDone() const;
  //const Sample *getSamplePtr() {return sample;}
  //const PlayingSample *getPtr() {return this;}
  void mix(jack_nframes_t nframes, jack_default_audio_sample_t *dest_buf, float gain);
};

#endif // PLAYINGSAMPLE_HPP
