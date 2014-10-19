from gi.repository import Gtk

import tdrum as tcore

class Fader(object):
    @classmethod
    def InitClass(cls, builder, gladefile):
        cls.builder = builder
        cls.gladefile = gladefile

    def __init__(self, fader_name, container, core):
        super(Fader, self).__init__()
        self.builder.add_objects_from_file(self.gladefile, ["fader_frame"])
        label = self.builder.get_object("fader_label")
        label.set_text(fader_name)
        fader = self.builder.get_object("fader_frame")

        level_adjustment = Gtk.Adjustment(1.0, 0.0, 1.0, 0.005, 0.0, 0.0)
        level_adjustment.connect("value-changed", self.set_level)
        level_scale = self.builder.get_object("level_scale")
        level_scale.set_adjustment(level_adjustment)

        self.core_fader = tcore.Fader(fader_name)
        core.addFader(self.core_fader)

        container.pack_start(fader, False, False, 0)
        fader.reparent(container)
        fader.show()

    def get_core_fader(self):
        return self.core_fader

    def set_level(self, adjustment):
        self.core_fader.setGain(adjustment.get_value())
