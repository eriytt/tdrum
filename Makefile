OBJECTS = tdrum_wrap.o TDrum.o Notify.o MidiMessage.o

PYTHON=python2.7

all: _tdrum.so
.PHONY : all

tdrum_wrap.cpp: tdrum.i
	swig -c++ -python -o $@ $< 


PYTHON_CXXFLAGS = -I/usr/include/$(PYTHON)
CXXFLAGS = -Wall -std=c++11 -MMD -fPIC $(PYTHON_CXXFLAGS)

PYTHON_LDFLAGS = -l$(PYTHON)
SNDFILE_LDFLAGS = -lsndfile
JACK_LDFLAGS = -ljack
LD_FLAGS = -shared $(PYTHON_LDFLAGS) $(SNDFILE_LDFLAGS) $(JACK_LDFLAGS)
_tdrum.so: $(OBJECTS)
	g++ $(LD_FLAGS) -o $@ $(OBJECTS) 

clean:
	rm $(OBJECTS) _tdrum.so
.PHONY: clean

-include $(OBJECTS:%.o=%.d)
