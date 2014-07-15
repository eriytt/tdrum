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
  return sample_data[frame];
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

bool Instrument::loadSample(const std::string &path)
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

  Sample *s = new Sample(data, items);
  addSample(s);
  return true;
}

const Sample *Instrument::getSampleForVelocity(unsigned char velocity)
{
  // TODO: dummy implementation
  return samples[0];
}

void Instrument::addSample(const Sample *sample)
{
  samples.push_back(sample);
}

void Core::addInstrument(unsigned short key, Instrument* instr)
{
  std::cout << "Adding instrument as key " << key << std::endl;
}

// sound engine methods
void Core::mixInstrument(unsigned short key, unsigned char velocity)
{
#warning This is not thread safe, suppose another thread is fiddling with instrument configuration.
  if (not keyToInstrument.count(key))
    return;

  Instrument *i = keyToInstrument[key];
  const Sample *s = i->getSampleForVelocity(velocity);
  playing_samples.push_back(PlayingSample(s)); // TODO: move semantics
}

void Core::mix(jack_nframes_t nframes, jack_default_audio_sample_t *dest_buf)
{
  for (jack_nframes_t i = 0; i < nframes; ++i)
    for (unsigned int s = playing_samples.size() - 1; s > 0; --s)
      {
	PlayingSample &sample = playing_samples[s];
	dest_buf[i] += sample.getNextFrame();
	playing_samples[i].getNextFrame();
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
  if ((jack_client = jack_client_open ("TDrum", JackNullOption, &open_status)) == 0)
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
      jack_midi_event_get(&e, midi_buf, ei);
      MidiMessage mm(e.buffer, e.size, e.time);

      // TODO, handle all types of messages
      if (not mm.is_note_on())
	continue;

      jack_nframes_t mixtime = e.time - last_event_time;
      if (mixtime)
	{
	  mix(mixtime, out);
	  last_event_time = e.time;
	}
      mixInstrument(mm.getP0(), mm.getP1());
    }
  return 0;
}
