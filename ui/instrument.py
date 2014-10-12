from gi.repository import Gtk

import fader
from utils import Utils

class Instrument(object):
    @classmethod
    def InitClass(cls, builder, gladefile):
        cls.builder = builder
        cls.builder.add_objects_from_file(gladefile, ["instrument_dialog"])
        cls.instrument_dialog = builder.get_object("instrument_dialog")
        cls.samples_treeview = builder.get_object("samples_treeview")
        cls.instrument_gain_scale = builder.get_object("instrument_gain_scale")
        cls.sample_gain_scale = builder.get_object("sample_gain_scale")

    @classmethod
    def CreateNewInstrument(cls, widget, container, sigproxy):
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

            return tmp_instr.finalize()

            
        return None

    def __init__(self, instrument_name, container):
        super(Instrument, self).__init__()
        self.name = instrument_name
        self.container = container
        self.sample_store = Gtk.ListStore(str, int, str, Gtk.Adjustment, Gtk.Adjustment, int)
        self.gain_adjustment = Gtk.Adjustment(1.0, 0.0, 1.0, 0.05, 0.0, 0.0)
        self.gain_adjustment.connect("value-changed", self.set_gain)
        self.gain = self.gain_adjustment.get_value()
        self.fader = None

    def finalize(self):
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

    def set_gain(self, adjustment):
        self.gain = adjustment.get_value()

    def name_changed(self, widget):
        self.name = widget.get_text()

    def trig_level_edited(self, widget, path, value):
        self.sample_store[path][1] = int(value)

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
            self.sample_store.append([filename,
                                      0, # trig level
                                      filename, # displayed file
                                      Gtk.Adjustment(0, 0, 127, 1, 10, 0), # trig level adjustment
                                      sample_gain_adjustment,
                                      1.0,
                                  ])


        dialog.destroy()

    def activate_sample(self, treeview, path, column):
        self.sample_gain_scale.set_adjustment(self.sample_store[path][4])
        #print treeview, path, column


    def delete_sample(self, widget):
        print "Delete sample"
        
