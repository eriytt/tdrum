#include "Fader.hpp"

void Fader::mix(jack_nframes_t nframes, jack_default_audio_sample_t *dest_buf, float gain)
{
  if (mixed)
    return;

  if (downstream and (not dest_buf))
    downstream->mix(nframes, nullptr, 0);

  mixed = true;

  jack_default_audio_sample_t *out_buffer = dest_buffer;
  jack_default_audio_sample_t *other_buffer = nullptr;

  if (out_port) // TODO: needed?
    {
      out_buffer = static_cast<jack_default_audio_sample_t *>
	(jack_port_get_buffer(out_port, nframes));
      other_buffer = dest_buffer;
    }
  
  if (not out_buffer)
    return;

  std::fill_n(out_buffer, nframes, 0.0);

  for (auto s : sources)
    (*s)->mix(nframes, out_buffer, fader_gain * gain);
  
  if (other_buffer)
    memcpy(other_buffer, out_buffer, nframes * sizeof(jack_default_audio_sample_t));
}
