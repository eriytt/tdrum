#ifndef NOTIFY_HPP
#define NOTIFY_HPP

#include <string>

class Notify
{
public:
  typedef enum class _NotifierType
  {
    ERROR = 1,
    WARNING,
    INFO,
    VERBOSE,
    TRACE,
    DEBUG
  } NotifierType;

public:
  static void notify(NotifierType t, const std::string &what, const std::string &why);

private:
  static const Notify &singleton;
  Notify();
};

#endif // NOTIFY_HPP
