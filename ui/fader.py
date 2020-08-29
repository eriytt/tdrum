from gi.repository import Gtk

import ui.bus as bus
import ui.instrument as instrument

class Fader:
    @classmethod
    def InitClass(cls, builder, gladefile):
        cls.builder = builder
        #builder.add_objects_from_file(gladefile, ["fader_frame"])
        cls.gladefile = gladefile

    def __init__(self, fader_name, container, core_fader, widget_prefix):
        self.container = container
        builder = Gtk.Builder()
        builder.add_objects_from_file(self.gladefile, [widget_prefix + "fader_frame"])
        builder.connect_signals(self)

        def get_object(name):
            return builder.get_object(widget_prefix + name)

        self.label = get_object("fader_label")
        self.label.set_text(fader_name)
        self.fader_frame = get_object("fader_frame")

        level_adjustment = Gtk.Adjustment(1.0, 0.0, 1.0, 0.005, 0.0, 0.0)
        level_adjustment.connect("value-changed", self.set_level)
        level_scale = get_object("level_scale")
        level_scale.set_adjustment(level_adjustment)

        pan_adjustment = Gtk.Adjustment(0.5, 0.0, 1.0, 0.005, 0.0, 0.0)
        pan_adjustment.connect("value-changed", self.set_pan)
        get_object("panning_scale").set_adjustment(pan_adjustment)

        self.core_fader = core_fader #tcore.Fader(fader_name)

        self.menu = get_object("input_menu")

        container.pack_start(self.fader_frame, False, False, 0)
        self.fader_frame.reparent(container)
        self.fader_frame.show()

    def destroy(self):
        self.core_fader.delete()
        self.container.remove(self.fader_frame)
        self.fader_frame.destroy()

    def save(self):
        return {
            'level': self.core_fader.get_gain(),
            'panning': self.core_fader.get_panning()
        }

    def load(self, obj):
        self.core_fader.set_gain(obj['level'])
        self.core_fader.set_panning(obj['panning'])

    def get_name(self):
        return self.label.get_text()

    def get_core_fader(self):
        return self.core_fader

    def set_level(self, adjustment):
        self.core_fader.set_gain(adjustment.get_value())

    def set_pan(self, adjustment):
        self.core_fader.set_panning(adjustment.get_value())

    def fader_popup(self, *args, **kwargs):pass
    #    print(f"Popup {args} {kwargs}")

    def motion(self, *args, **kwargs):pass
    #    print(f"Motion {args} {kwargs}")

class InstrumentFader(Fader):
    def __init__(self, fader_name, container, instrument, core_fader):
        super(InstrumentFader, self).__init__(fader_name, container, core_fader, "")
        self.instrument = instrument

    def play_instrument(self, *args, **kwargs):
        self.instrument.play(127)

class BusFader(Fader):
    def __init__(self, fader_name, container, bus, core_fader):
        super(BusFader, self).__init__(fader_name, container, core_fader, "bus_")
        self.bus = bus

    def popup_inputs(self, menuitem):
        self.menu.forall(self.menu.remove)

        def add_to_menu(item):
            mi = Gtk.MenuItem(item.name)
            mi.item = item
            mi.connect('activate', self.bus.set_input, item)
            self.menu.append(mi)
            mi.show()

        buses = [b for b in sorted(bus.Bus.GetBuses().values(), key = lambda v: v.name)
                 if b not in self.bus.inputs]
        instruments = [i for i in sorted(instrument.Instrument.GetInstruments().values(), key = lambda v: v.name)
                       if i not in self.bus.inputs]

        for b in buses:
            add_to_menu(b)

        if buses and instruments:
            self.menu.append(Gtk.SeparatorMenuItem())

        for i in instruments:
            add_to_menu(i)
