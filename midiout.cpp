/*
  Copyright (C) 2004 Ian Esten

  This program is free software; you can redistribute it and/or modify
  it under the terms of the GNU General Public License as published by
  the Free Software Foundation; either version 2 of the License, or
  (at your option) any later version.

  This program is distributed in the hope that it will be useful,
  but WITHOUT ANY WARRANTY; without even the implied warranty of
  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
  GNU General Public License for more details.

  You should have received a copy of the GNU General Public License
  along with this program; if not, write to the Free Software
  Foundation, Inc., 675 Mass Ave, Cambridge, MA 02139, USA.
*/

#include <jack/jack.h>
#include <jack/midiport.h>

#include "MidiMessage.hpp"
class midiout
{

protected:
  jack_client_t *client;
  jack_port_t *output_port;
  MidiMessage midinote;

public:
  midiout() : client(nullptr), output_port(nullptr)
  {
    midinote.p0() = 0;
    midinote.p1() = 0;
    midinote.p2() = 0;
    midinote.p3() = 0;
  }

protected:
  static int JackProcessTrampoline(jack_nframes_t nframes, void *arg)
  {
    return static_cast<midiout*>(arg)->jackProcess(nframes);
  }

  int jackProcess(jack_nframes_t nframes)
  {

      void* port_buf = jack_port_get_buffer(output_port, nframes);
      jack_midi_clear_buffer(port_buf);

      if (not midinote.p0())
	return 0;

      unsigned char* buffer = jack_midi_event_reserve(port_buf, 0, 3);

      // buffer[0] = 0x90;   /* note on */
      // buffer[1] = 10;
      // buffer[2] = 64;     /* velocity */
      buffer[0] = midinote.getCommand();   /* note on */
      buffer[1] = midinote.getP1();
      buffer[2] = midinote.getP2();

      midinote.p1() = 0;
      midinote.p2() = 0;
      midinote.p0() = 0;

      return 0;
    }


public:
  bool connectJack()
  {
    if ((client = jack_client_open ("midiout", JackNoStartServer, 0)) == nullptr)
      return false;

    jack_set_process_callback (client, JackProcessTrampoline, this);
    output_port = jack_port_register (client, "out", JACK_DEFAULT_MIDI_TYPE, JackPortIsOutput, 0);

    if (jack_activate(client))
      return false;
    return true;
  }

  void playNote(unsigned char note, unsigned char velocity)
  {
    while(midinote.getCommand());

    midinote.setP1(note);
    midinote.setP2(velocity);
    midinote.setCommand(0x90);
  }
};
