#include "Fader.hpp"

#include <cstring>
#include <iostream>

void Fader::registerJackPorts(jack_client_t *client)
{
  out_port = jack_port_register (client, (name + "_out").c_str(), JACK_DEFAULT_AUDIO_TYPE, JackPortIsOutput, 0);
}

// bool source_is_done(FaderSource *src) {
//   if (!src->isDone()) return false;
//   delete src;
//   return true;
// }

void Fader::mix(jack_nframes_t nframes, jack_default_audio_sample_t *dest_buf, float gain)
{
  //std::cout << "Mixing " << name << std::endl;
  if (mixed)
    {
      //std::cout << name << " already mixed" << std::endl;
      return;
    }

  if (downstream and (not dest_buf))
    {
      //std::cout << name << " triggering mix on downstream " << downstream->getName() << std::endl;
      downstream->mix(nframes, nullptr, 1.0);
      return;
    }

  //std::cout << name << " is mixed" << std::endl;
  mixed = true;

  jack_default_audio_sample_t *out_buffer = dest_buf;
  jack_default_audio_sample_t *other_buffer = nullptr;

  if (out_port) // TODO: needed?
    {
      out_buffer = static_cast<jack_default_audio_sample_t *>
	(jack_port_get_buffer(out_port, nframes));
      std::fill_n(out_buffer, nframes, 0.0);

      other_buffer = dest_buf;
    }

  if (not out_buffer)
    return;

  for (auto s : sources)
    s->mix(nframes, out_buffer, fader_gain * gain);

  // delete and remove finished sources
  sources.remove_if([] (FaderSource *src)
		    {if (!src->isDone()) return false; delete src; return true;});
  //sources.remove_if(source_is_done);

  // for (auto s = sources.begin(); s != sources.end(); ++s)
  //   if ((*s)->isDone())
  //     {
  // 	delete *s;
  // 	sources.erase(s);
  //     }

  if (other_buffer)
    memcpy(other_buffer, out_buffer, nframes * sizeof(jack_default_audio_sample_t));
}
