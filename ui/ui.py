import json

import pprint

import ui.signalproxy as signalproxy
import ui.bus as bus
import ui.instrument as instrument
import ui.fader as fader
from ui.utils import Utils
from gi.repository import Gtk
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
            with open(filename) as fd:
                # TODO: catch errors
                channels = json.load(fd)
            self.channels = []
            container = self.builder.get_object("fader_box")
            for c in channels:
                #TODO: catch errors
                if c['type'] == 'Bus':
                    channel = bus.Bus.load(c, container)
                elif c['type'] == 'Instrument':
                    channel = instrument.Instrument.load(c, container)
                self.channels.append(channel)
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
            if not self.core.registerJack():
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
