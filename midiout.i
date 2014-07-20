%module midiout

%{
#include "midiout.cpp"
%}

class midiout
{
 public:
  bool connectJack();
  void playNote(unsigned char note, unsigned char velocity);
};
