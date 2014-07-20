#include "TDrum.hpp"

#include <iostream>
#include <sstream>
#include <algorithm>

#include <sndfile.h>

#include "MidiMessage.hpp"

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

PlayingSample::PlayingSample(const Sample *sample) : current_position(0), sample(sample)
{
  //std::cout << "New PlayingSample " << this << ", Sample : " << sample << std::endl;
}

inline jack_default_audio_sample_t PlayingSample::getNextFrame()
{
  if (isDone())
    return 0.0;
  return sample->getFrame(current_position++);
}

inline bool PlayingSample::isDone() const
{
  return current_position == sample->size();
}

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

void Core::addInstrument(unsigned short note, Instrument* instr)
{
  std::cout << "Adding instrument " << instr <<  " as note " << note << std::endl;
  noteToInstrument[note] = instr;
}

// sound engine methods
void Core::mixInstrument(unsigned short note, unsigned char velocity)
{
#warning This is not thread safe, suppose another thread is fiddling with instrument configuration.
  //std::cout << "Adding sample " << note << " at velocity " << static_cast<unsigned int>(velocity) << std::endl;
  if (not noteToInstrument.count(note))
    return;

  Instrument *i = noteToInstrument[note];
  const Sample *s = i->getSampleForVelocity(velocity);
  //std::cout << "Adding sample to play" << std::endl;
  playing_samples.push_back(PlayingSample(s)); // TODO: move semantics
}

void Core::mix(jack_nframes_t nframes, jack_default_audio_sample_t *dest_buf)
{
  // if (playing_samples.size())
  //   std::cout << playing_samples.size() << " samples to be played" << std::endl;

  for (jack_nframes_t i = 0; i < nframes; ++i)
    for (auto s: playing_samples)
      {
	//std::cout << "Playing frame from Sample " << s.getSamplePtr() << std::endl;
	dest_buf[i] += s.getNextFrame();
      }

  auto i = playing_samples.begin();
  while (i != playing_samples.end())
    if ((*i).isDone())
      i = playing_samples.erase(i);
    else
      ++i;
}


// Jack handling methods

bool Core::registerJack()
{
  jack_status_t open_status;
  if ((jack_client = jack_client_open ("TDrum", JackNoStartServer, &open_status)) == 0)
    {
      std::stringstream err;
      err << open_status;
      Notify::notify(Notify::NotifierType::ERROR, "connecting to jack", err.str().substr());
      return false;
    }

  //calc_note_frqs(jack_get_sample_rate (client));

  jack_set_process_callback (jack_client, Core::JackProcessTrampoline, this);

  //jack_set_sample_rate_callback (jack_client, srate, 0);

  //jack_on_shutdown (jack_client, jack_shutdown, 0);

  midi_input_port = jack_port_register (jack_client, "midi_in", JACK_DEFAULT_MIDI_TYPE, JackPortIsInput, 0);
  audio_output_port = jack_port_register (jack_client, "audio_out", JACK_DEFAULT_AUDIO_TYPE, JackPortIsOutput, 0);

  if (int jerr = jack_activate (jack_client))
    {
      std::stringstream err;
      err << jerr;
      Notify::notify(Notify::NotifierType::ERROR, "connecting to jack", err.str().substr());
      return false;
    }

  // /* run until interrupted */
  // while(1)
  //   {
  //     sleep(1);
  //   }
  // jack_client_close(jack_client);
  // exit (0);

  return true;
}


// Jack handling
int Core::JackProcessTrampoline(jack_nframes_t nframes, void *arg)
{
  Core *c = static_cast<Core*>(arg);
  return c->jackProcess(nframes);
}

inline int Core::jackProcess(jack_nframes_t nframes)
{
  jack_default_audio_sample_t *out = static_cast<jack_default_audio_sample_t *>
    (jack_port_get_buffer(audio_output_port, nframes));
  void* midi_buf = jack_port_get_buffer(midi_input_port, nframes);
  jack_nframes_t nevents = jack_midi_get_event_count(midi_buf);


  // TODO: is this necessary?
  // Clear the output buffer
  std::fill_n(out, nframes, 0.0);

  if (nevents == 0)
    {
      mix(nframes, out);
      return 0;
    }


  jack_midi_event_t e;
  jack_nframes_t last_event_time = 0;

  for(uint32_t ei = 0; ei < nevents; ++ei)
    {
      // std::cout << "Got midi event" << std::endl;
      jack_midi_event_get(&e, midi_buf, ei);
      MidiMessage mm(e.buffer, e.size, e.time);

      // TODO, handle all types of messages
      if (not mm.is_note_on())
	continue;

      // std::cout << "Got note on event" << std::endl;
      jack_nframes_t mixtime = e.time - last_event_time;
      if (mixtime)
	{
	  mix(mixtime, &out[last_event_time]);
	  last_event_time = e.time;
	}
      mixInstrument(mm.getP1(), mm.getP2());
    }

  mix(nframes - last_event_time, &out[last_event_time]);

  return 0;
}
