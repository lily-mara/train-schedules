select stop_name, departure_time, route_long_name, direction_id, stops.stop_id
from trips
join stop_times on stop_times.trip_id = trips.trip_id
join routes on routes.route_id = trips.route_id
join stops on stops.stop_id = stop_times.stop_id
where service_id='c_16869_b_19500_d_31' and direction_id=1
order by stops.stop_name, direction_id, departure_time;

select trip_id from stop_times where stop_id=70171
intersect
select trip_id from stop_times where stop_id=70111;

select departure_time from stop_times where stop_id=?;

select distinct stop_name, stops.stop_id, direction_id
from trips
join stop_times on stop_times.trip_id = trips.trip_id
join stops on stops.stop_id = stop_times.stop_id
where stops.stop_id=70171;


select departure_minute, arrival_minute, trip_id, stop_id
from stop_times
where (stop_id=70171 or stop_id=70111) and trip_id=803 and departure_minute > 595
order by trip_id;

select direction_id, count(*)
from trips
join (
select trip_id from stop_times where stop_id=70171
intersect
select trip_id from stop_times where stop_id=70111 ) as f on f.trip_id = trips.trip_id
group by direction_id;

select distinct stop_name, stops.stop_id, direction_id
from trips
join stop_times on stop_times.trip_id = trips.trip_id
join stops on stops.stop_id = stop_times.stop_id
where stops.stop_id=?;

