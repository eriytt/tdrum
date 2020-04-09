from gi.repository import GObject
from gi.repository import Gtk
import collections

import pprint
pp = pprint.PrettyPrinter()

def connect_signals(toplevel, receiver):
    return SignalProxy.instance.connect_signals(toplevel, receiver)

class SignalProxy:
    class SignalConnection:
        def __init__(self, signal_name, handler_name, handler_id, after):
            self.signal_name = signal_name
            self.handler_name = handler_name
            self.handler_id = handler_id
            self.after = after

    @classmethod
    def InitClass(cls, builder):
        cls.instance = SignalProxy(builder)

    @staticmethod
    def extract_handler_and_args(obj_or_map, handler_name):
        handler = None
        callable = lambda c: hasattr(c, '__call__')

        if isinstance(obj_or_map, collections.Mapping):
            handler = obj_or_map.get(handler_name, None)
        else:
            handler = getattr(obj_or_map, handler_name, None)

        if handler is None:
            raise AttributeError('Handler %s not found' % handler_name)

        args = ()
        if isinstance(handler, collections.Sequence):
            if len(handler) == 0:
                raise TypeError("Handler %s tuple can not be empty" % handler)
            args = handler[1:]
            handler = handler[0]

        elif not callable(handler):
            raise TypeError('Handler %s is not a method, function or tuple' % handler)

        return handler, args

    def __init__(self, builder):
        self.signal_map = {}
        builder.connect_signals_full(self.full_callback,  self)


    def full_callback(self, builder, gobj, signal_name, handler_name, connect_obj, flags, obj_or_map):
        dummy_handler = getattr(self, self.dummy_callback.__name__, None)

        after = flags & GObject.ConnectFlags.AFTER

        if after:
            handler_id = gobj.connect_object_after(signal_name, dummy_handler, connect_obj)
        else:
            handler_id = gobj.connect_object(signal_name, dummy_handler, connect_obj)

        self.signal_map.setdefault(gobj, []).append(SignalProxy.SignalConnection(
            signal_name, handler_name, handler_id, after))

    def connect_signals(self, toplevel, receiver):

        def get_widget_children(widget):
            has_method = lambda w, m: hasattr(w, m)
            children = []

            if isinstance(widget, Gtk.MenuItem):
                if widget.get_submenu():
                    children.append(widget.get_submenu())
            elif isinstance(widget, Gtk.TreeView):
                children.extend(widget.get_columns())
            elif isinstance(widget, Gtk.TreeViewColumn):
                children.extend(widget.get_cells())

            elif isinstance(widget, Gtk.Bin):
                children.append(widget.get_child())
            elif isinstance(widget, Gtk.Container):
                children.extend(widget.get_children())

            # print "Children of %s:" % str(widget)
            # pp.pprint(children)

            return children


        def connect_recurse(widget, receiver):
            if widget in self.signal_map:
                for conn in self.signal_map[widget]:
                    (handler, args) = self.extract_handler_and_args(receiver, conn.handler_name)

                    widget.disconnect(conn.handler_id)

                    if conn.after:
                        conn.handler_id = widget.connect_object_after(
                            conn.signal_name, handler, widget, *args)
                    else:
                        conn.handler_id = widget.connect_object(
                            conn.signal_name, handler, widget, *args)

            for c in get_widget_children(widget):
                connect_recurse(c, receiver)

        connect_recurse(toplevel, receiver)

    def dummy_callback(self, *args):
        raise Exception("Unconnected callback: %s" % ", ".join([str(a) for a in args]))
