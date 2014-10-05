from gi.repository import Gtk

class Utils(object):
    @classmethod
    def InitClass(cls, builder, gladefile, window):
        cls.builder = builder
        cls.gladefile = gladefile
        cls.topwindow = window

    @classmethod
    def popupdialog(cls, msgtype, buttons, msg1, msg2, flags = 0):
        dialog = Gtk.MessageDialog(cls.topwindow, flags, msgtype, buttons, msg1)
        dialog.format_secondary_text(msg2)
        res = dialog.run()  
        dialog.destroy()
        return res
    
    @classmethod
    def error(cls, msg1, msg2):
        cls.popupdialog(Gtk.MessageType.ERROR, Gtk.ButtonsType.OK, msg1, msg2, flags = Gtk.DialogFlags.DESTROY_WITH_PARENT)
