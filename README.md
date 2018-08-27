# `cpu-usage`

This library provides methods for getting the percentage of cpu time spent
idle, and the averaged cpu speed over a given period. It follows the structure
of `std::time`, in that you can only really work with a difference between 2
fixed points in time. Cpu usage and a specific instant is meaningless, it is
defined as the proportion of cpu cycles spent not idle over a given period.
