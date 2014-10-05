
class Fader(object):
    @classmethod
    def InitClass(cls, builder, gladefile):
        cls.builder = builder
        cls.gladefile = gladefile

    def __init__(self, fader_name, container):
        super(Fader, self).__init__()
        self.builder.add_objects_from_file(self.gladefile, ["fader_frame"])
        label = self.builder.get_object("fader_label")
        label.set_text(fader_name)
        fader = self.builder.get_object("fader_frame")

        container.pack_start(fader, False, False, 0)
        fader.reparent(container)
        fader.show()

