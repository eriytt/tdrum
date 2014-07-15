//
// Copyright 1997 by Craig Stuart Sapp, All Rights Reserved.
// Programmer: Craig Stuart Sapp <craig@ccrma.stanford.edu>
// Creation Date: 19 December 1997
// Last Modified: Fri Jan 23 00:21:24 GMT-0800 1998
// Last Modified: Sun Sep 20 20:30:53 PDT 1998
// Last Modified: Mon Oct 15 14:29:12 PDT 2001 (added is_note functions)
// Portions Copyright 2014 Erik Ytterberg
//
// Description: A structure for handling MIDI input messages.
// This class stores a time stamp plus up to
// four MIDI message bytes. System Exclusive messages
// are stored in a separate array in the MidiInPort
// class, and their storage index is passed to the
// user through a MIDI message for later extraction
// of the full sysex message.
//

#ifndef _MIDIMESSAGE_H_INCLUDED
#define _MIDIMESSAGE_H_INCLUDED

#include <unistd.h>

typedef unsigned char uchar;
typedef unsigned long ulong;

struct MidiMessage {
  ulong time; // timestamp
  ulong data; // MIDI command and parameters

  MidiMessage (void);
  MidiMessage (uchar aCommand, uchar aP1, uchar aP2, int aTime = 0);
  MidiMessage (uchar *buf, size_t size, int aTime = 0);
  MidiMessage (const MidiMessage& aMessage);
  ~MidiMessage ();

  uchar& command (void);
  MidiMessage& operator= (const MidiMessage& aMessage);
  uchar& operator[] (int index);
  uchar& p0 (void);
  uchar& p1 (void);
  uchar& p2 (void);
  uchar& p3 (void);
  int getArgCount (void) const;
  int getParameterCount (void) const;

  uchar getCommand (void) const;
  uchar getP0 (void) const;
  uchar getP1 (void) const;
  uchar getP2 (void) const;
  uchar getP3 (void) const;

  void setCommand (uchar aCommand);
  void setData (uchar aCommand, uchar aP1 = 0, uchar aP2 = 0, uchar aP3 = 0);
  void setP0 (uchar aP0);
  void setP1 (uchar aP1);
  void setP2 (uchar aP2);
  void setP3 (uchar aP3);

  bool is_note (void);
  bool is_note_on (void);
  bool is_note_off (void);

};


//ostream& operator<<(ostream& out, MidiMessage& aMessage);


#endif /* _MIDIMESSAGE_H_INCLUDED */



// md5sum: 794ef1f84a961e75f24e7d5484ee1f6c MidiMessage.h [20050403]
