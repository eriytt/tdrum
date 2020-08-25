#!/usr/bin/env python3

from gi.repository import Gtk

import ui

if __name__ == "__main__":
    app = ui.TDrumUI("tdrum.glade")
    Gtk.main()
