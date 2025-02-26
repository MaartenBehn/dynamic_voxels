#ifndef _INTERVAL_GLSL_
#define _INTERVAL_GLSL_

#include "aabb.glsl"

#define MAX_INTERVAL_LIST 10

struct Interval {
    float t_min;
    float t_max;
};

Interval init_interval() {
    return Interval(FLOAT_POS_INF, FLOAT_NEG_INF);
}

struct IntervalList {
    Interval[MAX_INTERVAL_LIST] intervals;
    AABB[MAX_INTERVAL_LIST] aabbs;
    int len;
};

IntervalList init_interval_list() {
    Interval intervals[MAX_INTERVAL_LIST];
    AABB aabbs[MAX_INTERVAL_LIST];
    return IntervalList(intervals, aabbs, 0);
}

// From https://www.geeksforgeeks.org/search-insert-position-of-k-in-a-sorted-array/
int binary_search_interval_list(IntervalList list, float t)
{
    // Lower and upper bounds
    int start = 0;
    int end = list.len - 1;
    // Traverse the search space
    while (start <= end) {
        int mid = (start + end) / 2;

        if (list.intervals[mid].t_min < t) {
            start = mid + 1;
        } else {
            end = mid - 1;
        }
    }
    // Return insert position
    return end + 1;
}

IntervalList insert_into_list(IntervalList list, Interval interval, AABB aabb) {
    uint index = binary_search_interval_list(list, interval.t_min);

    for (uint i = list.len; i > index; i--) {
        list.intervals[i] = list.intervals[i - 1];
        list.aabbs[i] = list.aabbs[i - 1];
    }

    list.intervals[index] = interval;
    list.aabbs[index] = aabb;
    list.len += 1;

    return list;
}

#endif 
