#include "Sample.hpp"

// TODO: inline?
const Sample *RoundRobinSample::getNextSample()
{
  const Sample *s = (*this)[current_sample];
  current_sample = (current_sample + 1) % size();
  return s;
}
