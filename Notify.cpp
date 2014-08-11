#include "Notify.hpp"

#include <exception>
#include <iostream>
#include <sstream>

const Notify &Notify::singleton = Notify();

// Notify::Notify()
// {
//   if (singleton)
//     throw std::exception();

//   singleton = this;
// }

Notify::Notify()
{
}

void Notify::notify(NotifierType t, const std::string &what, const std::string &why)
{
  std::string ts;

  switch (t)
    {
    case NotifierType::ERROR:
      ts = "error";
      break;
    case NotifierType::WARNING:
      ts = "warning";
      break;
    case NotifierType::VERBOSE:
      ts = "verbose";
      break;
    case NotifierType::TRACE:
      ts = "trace";
      break;
    case NotifierType::DEBUG:
      ts = "debug";
      break;

      // This can't happen we use a typesafe enum
    default:
      throw std::exception();
    }

  std::cout << ts << ", " << what << ": " << why << std::endl;
}
