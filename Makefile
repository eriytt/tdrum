OBJECTS = tdrum_wrap.o TDrum.o Notify.o MidiMessage.o \
          PlayingSample.o Sample.o  Instrument.o Fader.o

PYTHON=python2.7

all: _tdrum.so _midiout.so
.PHONY : all

tdrum_wrap.cpp: tdrum.i
	swig -c++ -python -o $@ $< 

midiout_wrap.cpp: midiout.i
	swig -c++ -python -o $@ $< 


DEBUG_CXXFLAGS = -g
OPTIMIZE_CXXFLAGS = -O2
PYTHON_CXXFLAGS = -I/usr/include/$(PYTHON)
CXXFLAGS = -Wall -std=c++11 -MMD -fPIC $(DEBUG_CXXFLAGS) $(PYTHON_CXXFLAGS)

PYTHON_LDFLAGS = -l$(PYTHON)
SNDFILE_LDFLAGS = -lsndfile
JACK_LDFLAGS = -ljack
LDFLAGS = -shared $(PYTHON_LDFLAGS) $(SNDFILE_LDFLAGS) $(JACK_LDFLAGS)
_tdrum.so: $(OBJECTS)
	g++ $(LDFLAGS) -o $@ $(OBJECTS) 

midiout_wrap.o: midiout.cpp

_midiout.so: midiout_wrap.o
	g++ -shared $(JACK_LDFLAGS) $(PYTHON_LDFLAGS) MidiMessage.o -o $@ $< 	

clean:
	rm -f $(OBJECTS) _tdrum.so tdrum_wrap.cpp
.PHONY: clean



-include $(OBJECTS:%.o=%.d)
