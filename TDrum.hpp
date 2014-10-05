#ifndef TDRUM_HPP
#define TDRUM_HPP
#include <map>
#include <vector>
#include <list>

#include <jack/jack.h>
#include <jack/midiport.h>

// class SampleRegistry
// {
// protected:
//   std::map<Sample *, Sample *> samples;
// };

//#include "Fader.hpp"

class Fader;
class Instrument;
class PlayingSample;

class Core
{
protected:
  std::map<unsigned short, Instrument*> noteToInstrument;
  std::list<Fader *> faders;
  // TODO: vector is a bad choice here because erase is very inefficient.
  //std::list<PlayingSample> playing_samples;

  //SampleRegistry registry;


  // Jack stuff
protected:
  jack_client_t *jack_client;
  jack_port_t *midi_input_port;
  jack_port_t *audio_output_port;
  static int JackProcessTrampoline(jack_nframes_t nframes, void *arg);
  int jackProcess(jack_nframes_t nframes);

protected:
  // Sound engine stuff
  void mixInstrument(unsigned short note, unsigned char velocity, int offset);
  void mix(jack_nframes_t nframes, jack_default_audio_sample_t *dest_buf);

public:
  Core(): jack_client(nullptr), midi_input_port(nullptr), audio_output_port(nullptr) {}
  void addInstrument(unsigned short note, Instrument* instr);
  void addFader(Fader *fader) {faders.push_back(fader);}
  jack_client_t *registerJack();
};

#endif // TDRUM_HPP
