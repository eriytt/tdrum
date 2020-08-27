import json

import pprint

import ui.signalproxy as signalproxy
import ui.bus as bus
import ui.instrument as instrument
import ui.fader as fader
from ui.utils import Utils
from gi.repository import Gtk
import ui.core as core
from ui.core import CoreInstrument, CoreFader

pp = pprint.PrettyPrinter()

class TDrumUI(object):
    def __init__(self, gladefile):
        #gtk.gdk.threads_init()

        self.channels = []

        self.gladefile = gladefile
        self.builder = Gtk.Builder()

        self.builder.add_objects_from_file(self.gladefile, ["main_window"])
        bus.Bus.InitClass(self.builder, self.gladefile)
        instrument.Instrument.InitClass(self.builder, self.gladefile)
        fader.Fader.InitClass(self.builder, self.gladefile)
        signalproxy.SignalProxy.InitClass(self.builder)

        self.win = self.builder.get_object("main_window")
        signalproxy.connect_signals(self.win, self)

        Utils.InitClass(self.builder, self.gladefile, self.win)

        self.win.connect("delete-event", Gtk.main_quit)
        self.win.set_title("TDrum")

        container = self.builder.get_object("fader_box")
        bus.Bus.CreateMaster(container)

        self.win.show_all()


    def new_bus(self, widget):
        container = self.builder.get_object("fader_box")
        b = bus.Bus.CreateNewBus(widget, container)
        self.channels.append(b)
        return True

    def new_instrument(self, widget):
        print("New instrument")
        container = self.builder.get_object("fader_box")
        i = instrument.Instrument.CreateNewInstrument(widget, container)
        self.channels.append(i)
        return True

    def nofader_popup(self, widget, event):
        return True

    def fader_popup(self, widget, event):
        print("fader popup", widget.get_name())
        return True

    def open(self, widget):
        dialog = Gtk.FileChooserDialog(title="Open",
                                       parent=self.win,
                                       action=Gtk.FileChooserAction.OPEN,
                                       buttons=(Gtk.STOCK_CANCEL, Gtk.ResponseType.CANCEL,
                                                Gtk.STOCK_OPEN, Gtk.ResponseType.OK))


        response = dialog.run()
        if response == Gtk.ResponseType.OK:
            filename = dialog.get_filename()
            dialog.hide()

            with open(filename) as fd:
                try:
                    channels = json.load(fd)
                except json.decoder.JSONDecodeError as e:
                    dialog.destroy()
                    Utils.error("Cannot load file", str(e))
                    return

            for b in [bus.Bus.master] + list(bus.Bus.buses.values()):
                for i in b.get_inputs():
                    b.del_input(i)

            for b in list(bus.Bus.buses.values()):
                b.destroy()
            bus.Bus.buses.clear()

            for i in list(instrument.Instrument.instruments.values()):
                i.destroy()
            instrument.Instrument.instruments.clear()


            self.channels = []
            inputs = []
            container = self.builder.get_object("fader_box")
            for c in channels:
                #TODO: catch errors
                if c['type'] == 'Bus':
                    if c['name'] != "Master":
                        channel = bus.Bus.load(c, container)
                    else:
                        channel = bus.Bus.master
                    inputs.append((channel, c['inputs']))
                elif c['type'] == 'Instrument':
                    channel = instrument.Instrument.load(c, container)
                self.channels.append(channel)


            busses_and_instruments = {
                **bus.Bus.GetBuses(),
                **instrument.Instrument.GetInstruments()
            }

            for (b, inputs) in inputs:
                for i in inputs:
                    b.set_input(None, busses_and_instruments[i])

        dialog.destroy()

    def save_as(self, widget):
        channels = [c.save() for c in self.channels]

        dialog = Gtk.FileChooserDialog(title="Save as",
                                       parent=self.win,
                                       action=Gtk.FileChooserAction.SAVE,
                                       buttons=(Gtk.STOCK_CANCEL, Gtk.ResponseType.CANCEL,
                                                Gtk.STOCK_SAVE, Gtk.ResponseType.OK))

        response = dialog.run()
        if response == Gtk.ResponseType.OK:
            filename = dialog.get_filename()
            with open(filename, 'w') as fd:
                # TODO: catch errors
                json.dump(channels, fd, indent = 4)
        dialog.destroy()

    # TODO: remove this eventually
    def motion(self, widget, event):
        return False
        print("in", widget.get_name())
        return True

    def connect_jack(self, checkmenuitem):
        if checkmenuitem.get_active():
            try:
                core.register_jack();
            except:
                Utils.error("Failed to connect to jack", "TODO: query error reason")
                checkmenuitem.handler_block_by_func(self.connect_jack)
                checkmenuitem.set_active(False)
                checkmenuitem.handler_unblock_by_func(self.connect_jack)
        else:
            self.core.unregisterJack()
        return True

if __name__ == "__main__":
    app = TDrumUI()
    Gtk.main()
