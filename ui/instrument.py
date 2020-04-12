import os
from gi.repository import Gtk

import ui.signalproxy as signalproxy
import ui.fader as fader
from ui.utils import Utils
import ui.core as core

import tdrum


class Instrument:
    @classmethod
    def InitClass(cls, builder, gladefile):
        cls.builder = builder
        cls.builder.add_objects_from_file(gladefile, ["instrument_dialog"])
        cls.instrument_dialog = builder.get_object("instrument_dialog")
        cls.samples_treeview = builder.get_object("samples_treeview")
        cls.instrument_gain_scale = builder.get_object("instrument_gain_scale")
        cls.sample_gain_scale = builder.get_object("sample_gain_scale")
        cls.midi_note_spinbutton = builder.get_object("midi_note_spinbutton")
        cls.instruments = {}

    @classmethod
    def GetInstruments(cls):
        return cls.instruments

    @classmethod
    def CreateNewInstrument(cls, widget, container):
        instr = Instrument(None, container)
        instr.setup_dialog(cls.instrument_dialog)

        # TODO: Enter should exit dialog with OK, Esc exit with CANCEL ?
        response = cls.instrument_dialog.run()
        while response == Gtk.ResponseType.OK:
            def err(reason):
                Utils.error("Cannot add instrument", reason)
                return cls.instrument_dialog.run()

            if instr.note in cls.instruments:
                response = err("Note is already taken by another instrument")
                continue

            if instr.name in [i.name for i in cls.instruments.values()]:
                response = err("Name is already taken by another instrument")
                continue

            valid_err = instr.is_valid()
            if valid_err:
                response = err(valid_err)
                continue

            cls.instrument_dialog.hide()
            cls.instruments[instr.note] = instr
            instr.finalize()
            return instr

        cls.instrument_dialog.hide()
        return None

    @classmethod
    def load(cls, obj, container):
        instrument = Instrument(obj['name'], container)
        #TODO: Load samples
        instrument.finalize()
        self.fader.load(obj['fader'])
        return instrument

    def __init__(self, instrument_name, container):
        self.name = instrument_name
        self.container = container
        self.sample_store = Gtk.ListStore(str, int, str, Gtk.Adjustment, Gtk.Adjustment, int, object)
        self.gain_adjustment = Gtk.Adjustment(1.0, 0.0, 1.0, 0.05, 0.0, 0.0)
        self.gain_adjustment.connect("value-changed", self.set_gain)

        self.note = 0
        self.midi_note_adjustment = Gtk.Adjustment(0, 0, 127, 1, 0.0, 0.0)
        self.midi_note_adjustment.connect("value-changed", self.set_note)

        self.gain = self.gain_adjustment.get_value()

        self.core_instrument = core.CoreInstrument(self.name or "")

        self.fader = None
        self.core_fader = None


    def save(self):
        samples = []
        for s in self.sample_store:
            samples.append({
                'filename': s,
                'trig_level': self.sample_store[s][1] 
            })

        i = {
            'type': 'Instrument',
            'name': self.fader.get_name(),
            'note': self.note,
            'samples': samples
        }

        i.update(self.fader.save())
        return b


    def finalize(self):
        for s in self.sample_store:
            sample = s[6]
            #path = s[0]
            self.core_instrument.add_sample(sample)

        self.fader = fader.InstrumentFader(
            self.name, self.container, self,
            core.CoreFader(self.name, ref=self.core_instrument.get_fader())
        )
        #self.core_instrument.setFader(self.fader.get_core_fader())
        return self

    def play(self, velocity):
        print(f"Playing note {self.note}")
        self.core.play_instrument(self.note, velocity)

    def is_valid(self):
        if not len(self.sample_store):
            return "Instrument must have at least one sample"
        if not self.name:
            return "Name not set"

        # TODO: check for name duplication
        return None

    def setup_dialog(self, dialog):
        signalproxy.connect_signals(dialog, self)
        self.name = self.builder.get_object("instrument_name_entry").get_text()
        self.samples_treeview.set_model(self.sample_store)
        self.instrument_gain_scale.set_adjustment(self.gain_adjustment)
        self.midi_note_spinbutton.set_adjustment(self.midi_note_adjustment)

    def set_gain(self, adjustment):
        self.gain = adjustment.get_value()

    def set_note(self, adjustment):
        self.note = int(adjustment.get_value())
        self.core_instrument.set_note(self.note)

    def name_changed(self, widget):
        self.name = widget.get_text()

    def trig_level_edited(self, widget, path, value):
        print("Setting trig level:", int(value))
        self.sample_store[path][1] = int(value)
        sample = self.sample_store[path][6]
        sample.trig = int(value)
        #self.core_instrument.setVelocity(self.sample_store[path][6], int(value))

    def add_sample(self, widget):
        dialog = Gtk.FileChooserDialog("Choose an audio sample file",
                                       self.instrument_dialog,
                                       Gtk.FileChooserAction.OPEN,
                                       (Gtk.STOCK_CANCEL, Gtk.ResponseType.CANCEL,
                                        Gtk.STOCK_OPEN, Gtk.ResponseType.OK))

        #self.add_filters(dialog)

        response = dialog.run()
        if response == Gtk.ResponseType.OK:
            filename = dialog.get_filename()
            sample_gain_adjustment = Gtk.Adjustment(1.0, 0.0, 1.0, 0.05, 0.0, 0.0)
            self.sample_gain_scale.set_adjustment(sample_gain_adjustment)
            core_sample = tdrum.load_sample(filename)
            self.sample_store.append([filename,
                                      0, # trig level
                                      os.path.basename(filename), # displayed file
                                      Gtk.Adjustment(0, 0, 127, 1, 10, 0), # trig level adjustment
                                      sample_gain_adjustment,
                                      1.0,
                                      core_sample,
                                  ])



        dialog.destroy()

    def activate_sample(self, treeview, path, column):
        self.sample_gain_scale.set_adjustment(self.sample_store[path][4])
        #print treeview, path, column


    def delete_sample(self, widget):
        print("Delete sample")

