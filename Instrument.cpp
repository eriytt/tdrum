#include "Instrument.hpp"

#include <sstream>
#include <iostream>
#include <sndfile.h>

#include "Notify.hpp"

bool Instrument::loadSample(const std::string &path, unsigned char velocity)
{
  SF_INFO info;

  SNDFILE *fh = sf_open(path.c_str(), SFM_READ, &info);
  if (not fh)
    {
      Notify::notify(Notify::NotifierType::ERROR, std::string("opening file ") +  path, sf_strerror(fh));
      return false;
    }

  // TODO: what shall we do with multiple channels, we can only use
  // one
  sf_count_t items = info.frames * info.channels;
  float *data = new float[items];
  sf_count_t read_items = sf_read_float(fh, data, items);
  if (read_items != items)
    {
      std::stringstream err;
      err << "only " << read_items << " samples read out of " << items;
      Notify::notify(Notify::NotifierType::ERROR, std::string("reading file ") +  path, err.str().substr());
      return false;
    }

  sf_close(fh);

  std::cout << "Adding sample " << path << ", data: " << data << std::endl;
  Sample *s = new Sample(data, items);
  addSample(s, velocity);
  return true;
}

const Sample *Instrument::getSampleForVelocity(unsigned char velocity)
{
  // TODO: dummy implementation
  //std::cout << "getting sample " << samples[0] << std::endl;
  unsigned char vel = *velocities.begin();
  for (auto v : velocities)
    {
      if (v > velocity)
	break;
      vel = v;
    }

  // TODO: implement round robin
  return samples[vel].getNextSample();
}

void Instrument::addSample(const Sample *sample, unsigned char velocity)
{
  std::cout << "Adding Sample " << sample << " at index " << samples.size() << std::endl;
  if (samples.count(velocity) == 0)
    {
      velocities.push_back(velocity);
      velocities.sort();
    }

  samples[velocity].push_back(sample);
}
