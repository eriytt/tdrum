import pprint

import signalproxy
import bus
import instrument
import fader
import utils
from gi.repository import Gtk

pp = pprint.PrettyPrinter()

class TDrumUI(object):
    def __init__(self, gladefile):
        #gtk.gdk.threads_init()

        self.gladefile = gladefile
        self.builder = Gtk.Builder()

        self.builder.add_objects_from_file(self.gladefile, ["main_window"])
        self.win = self.builder.get_object("main_window")
        
        bus.Bus.InitClass(self.builder, self.gladefile)
        instrument.Instrument.InitClass(self.builder, self.gladefile)
        fader.Fader.InitClass(self.builder, self.gladefile)
        utils.Utils.InitClass(self.builder, self.gladefile, self.win)
        
        self.signal_proxy = signalproxy.SignalProxy(self.builder)
        self.signal_proxy.connect_signals(self.win, self)        

        self.win.connect("delete-event", Gtk.main_quit)
        self.win.set_title("TDrum")
        
        self.win.show_all()


    def new_bus(self, widget):
        container = self.builder.get_object("fader_box")
        bus.Bus.CreateNewBus(widget, container)
        return True

    def new_instrument(self, widget):
        container = self.builder.get_object("fader_box")
        instrument.Instrument.CreateNewInstrument(widget, container)
        return True

    def nofader_popup(self, widget, event):
        print "no fader popup", widget.get_name()
        return True

    def fader_popup(self, widget, event):
        print "fader popup", widget.get_name()
        return True

    # TODO: remove this eventually
    def motion(self, widget, event):
        return False
        print "in", widget.get_name()
        return True


if __name__ == "__main__":
    app = TDrumUI()
    Gtk.main()
