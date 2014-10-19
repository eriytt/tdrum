from gi.repository import Gtk

import fader
from utils import Utils

class Bus(object):
    @classmethod
    def InitClass(cls, builder, gladefile):
        cls.builder = builder
        cls.builder.add_objects_from_file(gladefile, ["new_bus_dialog"])
        cls.new_bus_dialog = builder.get_object("new_bus_dialog")


    @classmethod
    def CreateNewBus(cls, widget, container, sigproxy, core):
        # TODO: Enter should exit dialog with OK, Esc exit with CANCEL
        response = cls.new_bus_dialog.run()
        cls.new_bus_dialog.hide()
        
        if response == Gtk.ResponseType.OK:
            entry = cls.builder.get_object("bus_name_entry")
            name = entry.get_text()
            # TODO: check for name duplication
            if name:
                return Bus(name, container, core)
            else:
                Utils.error("Cannot add bus", "Invalid bus name: '%s'" % name)
            
        return None


    def __init__(self, bus_name, container, core):
        super(Bus, self).__init__()
        self.fader = fader.Fader(bus_name, container, core)

        
