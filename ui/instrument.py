import os
from gi.repository import Gtk

import fader
from utils import Utils
import tdrum as tcore

class Instrument(object):
    @classmethod
    def InitClass(cls, builder, gladefile):
        cls.builder = builder
        cls.builder.add_objects_from_file(gladefile, ["instrument_dialog"])
        cls.instrument_dialog = builder.get_object("instrument_dialog")
        cls.samples_treeview = builder.get_object("samples_treeview")
        cls.instrument_gain_scale = builder.get_object("instrument_gain_scale")
        cls.sample_gain_scale = builder.get_object("sample_gain_scale")
        cls.midi_note_spinbutton = builder.get_object("midi_note_spinbutton")

    @classmethod
    def CreateNewInstrument(cls, widget, container, sigproxy, core):
        tmp_instr = Instrument(None, container)
        tmp_instr.setup_dialog(cls.instrument_dialog, sigproxy)

        # TODO: Enter should exit dialog with OK, Esc exit with CANCEL ?
        response = cls.instrument_dialog.run()
        cls.instrument_dialog.hide()
        
        if response == Gtk.ResponseType.OK:
            (valid, errmsg) = tmp_instr.is_valid()
            if not valid:
                Utils.error("Cannot add instrument", errmsg)
                return None

            return tmp_instr.finalize(core)

            
        return None

    def __init__(self, instrument_name, container):
        super(Instrument, self).__init__()
        self.name = instrument_name
        self.container = container
        self.sample_store = Gtk.ListStore(str, int, str, Gtk.Adjustment, Gtk.Adjustment, int, object)
        self.gain_adjustment = Gtk.Adjustment(1.0, 0.0, 1.0, 0.05, 0.0, 0.0)
        self.gain_adjustment.connect("value-changed", self.set_gain)
        
        self.note = 0
        self.midi_note_adjustment = Gtk.Adjustment(0, 0, 127, 1, 0.0, 0.0)
        self.midi_note_adjustment.connect("value-changed", self.set_note)

        self.gain = self.gain_adjustment.get_value()

        self.core = None

        self.fader = None
        self.core_fader = None

        self.core_instrument = tcore.Instrument()

    def finalize(self, core):
        self.core = core
        self.core.addInstrument(self.note, self.core_instrument)
        self.core_fader = tcore.Fader(self.name)
        core.addFader(self.core_fader)
        self.core_instrument.setFader(self.core_fader)
        self.fader = fader.Fader(self.name, self.container)
        return self

    def is_valid(self):
        if not len(self.sample_store):
            return (False, "Instrument must have at least one sample")
        if not self.name:
            return (False, "Name not set")

        # TODO: check for name duplication
        return (True, "OK")

    def setup_dialog(self, dialog, sigproxy):
        sigproxy.connect_signals(dialog, self)
        self.samples_treeview.set_model(self.sample_store)
        self.instrument_gain_scale.set_adjustment(self.gain_adjustment)
        self.midi_note_spinbutton.set_adjustment(self.midi_note_adjustment)

    def set_gain(self, adjustment):
        self.gain = adjustment.get_value()

    def set_note(self, adjustment):
        self.note = int(adjustment.get_value())
        if self.core:
            core.setInstrumentNote(self.note, self.core_instrument)

    def name_changed(self, widget):
        self.name = widget.get_text()

    def trig_level_edited(self, widget, path, value):
        print "Setting trig level:", int(value)
        self.sample_store[path][1] = int(value)
        self.core_instrument.setVelocity(self.sample_store[path][6], int(value))

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
            core_sample = self.core_instrument.loadSample(filename, 0)
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
        print "Delete sample"
        
