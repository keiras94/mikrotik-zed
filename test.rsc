# Test RSC syntax highlighting
/ip address add address=10.0.0.1/24 interface=ether1

:global myVar "hello"
:local counter 0

:if ($counter < 10) do={
  :put "Counter: $counter"
  :set counter ($counter + 1)
}

/ip firewall filter
add chain=input action=accept protocol=tcp dst-port=22 comment="Allow SSH"

:foreach route in=[/ip route find] do={
  :put [/ip route get $route gateway]
}

:return true
