from gi.repository import Gtk

import ui.fader as fader
from ui.utils import Utils
import ui.core as core

class Bus(object):
    @classmethod
    def InitClass(cls, builder, gladefile):
        cls.builder = builder
        cls.builder.add_objects_from_file(gladefile, ["new_bus_dialog"])
        cls.new_bus_dialog = builder.get_object("new_bus_dialog")
        cls.buses = {}
        cls.master = None

    @classmethod
    def GetBuses(cls):
        return cls.buses

    @classmethod
    def CreateMaster(cls, container):
        master = Bus()
        master.name = "Master"
        master.fader = fader.BusFader(master.name, container, master, core.get_master_fader())
        cls.master = master
        return master

    def RebuildSignalChain(cls):
        pass


    @classmethod
    def CreateNewBus(cls, widget, container):

        name_entry = cls.builder.get_object("bus_name_entry")
        name_entry.set_text("")

        response = cls.new_bus_dialog.run()
        while response == Gtk.ResponseType.OK:
            def err(reason):
                Utils.error("Cannot add bus", reason)
                return cls.new_bus_dialog.run()

            name = name_entry.get_text()
            if name in [b.name for b in cls.buses.values()]:
                response = err("Name is already taken by another bus")
                continue

            bus = Bus().finalize(name, container)
            cls.new_bus_dialog.hide()
            cls.buses[bus.name] = bus
            #core.add_bus(bus.name, bus.core_fader)
            return bus

        cls.new_bus_dialog.hide()
        return None

    @classmethod
    def load(cls, obj, container):
        bus = Bus().finalize(obj['name'], container)
        bus.fader.load(obj['fader'])
        return bus

    def __init__(self):
        self.inputs = []

    def finalize(self, name, container):
        self.name = name
        self.fader = fader.BusFader(self.name, container, self, core.CoreFader(name))
        return self

    def save(self):
        return  {
            'type': 'Bus',
            'name': self.fader.get_name(),
            'fader': self.fader.save()
        }

    def set_input(self, menuitem, bus_or_instrument):
        self.inputs.append(bus_or_instrument)
        self.fader.get_core_fader().add_fader_src(bus_or_instrument.fader.get_core_fader())
