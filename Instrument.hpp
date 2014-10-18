#ifndef INSTRUMENT_HPP
#define INSTRUMENT_HPP

#include <map>
#include <list>

#include "Sample.hpp"

class Fader;

class Instrument
{
protected:
  std::map< unsigned char, RoundRobinSample > samples;
  std::list<unsigned char> velocities;
  Fader *fader;

protected:
  void addSample(const Sample *sample, unsigned char velocity);

public:
  const Sample *loadSample(const std::string &path, unsigned char velocity);
  void setVelocity(const Sample *sample, unsigned char velocity);
  const Sample *getSampleForVelocity(unsigned char velocity);
  Fader *getFader() {return fader;}
  void setFader(Fader *f) {fader = f;}
};

#endif // INSTRUMENT_HPP
