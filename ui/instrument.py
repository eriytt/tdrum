from gi.repository import Gtk

import fader
from utils import Utils

class Instrument(object):
    @classmethod
    def InitClass(cls, builder, gladefile):
        cls.builder = builder
        cls.builder.add_objects_from_file(gladefile, ["instrument_dialog"])
        cls.instrument_dialog = builder.get_object("instrument_dialog")

    @classmethod
    def CreateNewInstrument(cls, widget, container, sigproxy):
        tmp_instr = Instrument(None, container)
        sigproxy.connect_signals(cls.instrument_dialog, tmp_instr)

        # TODO: Enter should exit dialog with OK, Esc exit with CANCEL
        response = cls.instrument_dialog.run()
        cls.instrument_dialog.hide()
        
        if response == Gtk.ResponseType.OK:
            #entry = cls.builder.get_object("instrument_name_entry")
            #name = entry.get_text()
            # TODO: check for name duplication
            #if name:
            # return Instrument(name, container)
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
        self.fader = None

        
        # store = Gtk.ListStore(int, str)
        # store.append([64, "Sample 1"])
        # store.append([65, "Sample 2"])

        # renderer_spin = Gtk.CellRendererSpin()
        # renderer_spin.set_property("editable", True)
        # adjustment = Gtk.Adjustment(0, 0, 127, 1, 10, 0)
        # renderer_spin.set_property("adjustment", adjustment)

        # tv = self.builder.get_object("samples_treeview")
        # tv.append_column(Gtk.TreeViewColumn("Trig Level", renderer_spin, text = 0))
        # tv.append_column(Gtk.TreeViewColumn("Sample File", Gtk.CellRendererText(), text = 1))

        # tv.set_model(store)

    def finalize(self):
        self.fader = fader.Fader(self.name, self.container)
        return self

    def is_valid(self):
        return (False, "Validation not implemented")
