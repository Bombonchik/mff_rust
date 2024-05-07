use std::sync::{Arc, Mutex};
use std::collections::{HashSet, HashMap, VecDeque, BTreeMap};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct City {
    name: String
}

impl City {
    pub fn name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Road {
    travel_time: u32,
    point_a: Arc<City>,
    point_b: Arc<City>,
}

pub struct Bus {
    id: u32,
    route: Mutex<VecDeque<Arc<City>>>,
    upcoming_stops: Mutex<HashSet<Arc<City>>>,
    //total_route: VecDeque<Arc<City>>,
    time_people_getting_off: Mutex<BTreeMap<Arc<City>, u32>>,
    finished: Mutex<bool>,
}

impl Bus {
    pub fn new(route: Vec<Arc<City>>, id: u32) -> Self {
        let route_deque = VecDeque::from(route.to_vec());
        let upcoming_stops = Mutex::new(route.iter().cloned().collect());
        Bus {
            id,
            route: Mutex::new(route_deque.clone()),
            upcoming_stops,
            //total_route: route_deque,
            time_people_getting_off: Mutex::new(BTreeMap::new()),
            finished: Mutex::new(false),
        }
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn is_upcoming_stop(&self, city: Arc<City>) -> bool {
        self.upcoming_stops.lock().unwrap().contains(&city) && city != self.current_stop()
    }

    fn current_stop(&self) -> Arc<City> {
        self.route.lock().unwrap().front().unwrap().clone()
    }

    pub fn move_to_next(&self) {
        let mut finished = self.finished.lock().unwrap();
        if *finished {
            return;
        }
        let mut route = self.route.lock().unwrap();
        let mut upcoming_stops = self.upcoming_stops.lock().unwrap();

        if let Some(next_city) = route.pop_front() {
            upcoming_stops.remove(&next_city);
        } else {
            *finished = true;
        }
    }

    pub fn calculate_travel_time(&self, roads: &HashSet<Arc<Road>>, stop: Arc<City>, current_time: u32) -> u32 {
        let mut time_people_getting_off = self.time_people_getting_off.lock().unwrap();
        if let Some(&travel_time) = time_people_getting_off.get(&stop) {
            return travel_time;
        }
        let mut total_travel_time = current_time;
        let mut current_stop = self.current_stop();

        // Skipping the first city in the route as it's the current stop
        for city in self.route.lock().unwrap().iter().skip(1) {
            // Find the road between current_stop and the next city in the route
            if let Some(road) = roads.iter().find(|road| {
                (Arc::ptr_eq(&road.point_a, &current_stop) && Arc::ptr_eq(&road.point_b, city)) ||
                (Arc::ptr_eq(&road.point_a, city) && Arc::ptr_eq(&road.point_b, &current_stop))
            }) {
                total_travel_time += road.travel_time;

                // Check if we have reached the requested stop
                if Arc::ptr_eq(city, &stop) {
                    break;
                }
                current_stop = city.clone();
            }
        }
        time_people_getting_off.insert(stop.clone(), total_travel_time);
        total_travel_time
    }

}

#[derive(Clone)]
pub struct Event {
    bus: Arc<Bus>,
    city: Arc<City>,
    got_off_count: u32,
    got_on_count: u32,
}

impl Event {
    pub fn got_off(&self) -> u32 {
        self.got_off_count
    }

    pub fn got_on(&self) -> u32 {
        self.got_on_count
    }

    pub fn city(&self) -> &Arc<City> {
        &self.city
    }
}

pub struct Simulation {
    buses: Vec<Arc<Bus>>,
    roads: HashSet<Arc<Road>>,
    // Maps each city to a record of destinations and the number of people waiting to travel there.
    // For each city (key), it holds a map of destination cities (inner key) and passenger counts (value).
    waiting_people: HashMap<Arc<City>, HashMap<Arc<City>, u32>>,
    next_bus_id: u32,
    event_queue: BTreeMap<u32, BTreeMap<u32, Arc<Event>>>,
    current_time: u32,
}

impl Simulation {
    pub fn new() -> Self {
        Simulation {
            buses: Vec::new(),
            roads: HashSet::new(),
            waiting_people: HashMap::new(),
            next_bus_id: 0,
            event_queue: BTreeMap::new(),
            current_time: 0,
        }
    }

    pub fn new_city(&mut self, name: &str) -> Arc<City> {
        Arc::new(City {
            name: name.to_string()
        })
    }

    pub fn new_road(&mut self, a: &Arc<City>, b: &Arc<City>, travel_time: u32) -> Arc<Road> {
        let road = Arc::new(Road {
            travel_time,
            point_a: a.clone(),
            point_b: b.clone(),
        });
        self.roads.insert(road.clone());
        road
    }

    fn valid_route(&self, route: &Vec<Arc<City>>) {
        if route.len() < 2 {
            panic!("Invalid bus route: A bus must have at least two stops.");
        }

        let has_valid_roads = route.windows(2).all(|cities| {
            self.roads.iter().any(|road| 
                (Arc::ptr_eq(&road.point_a, &cities[0]) && Arc::ptr_eq(&road.point_b, &cities[1])) ||
                (Arc::ptr_eq(&road.point_a, &cities[1]) && Arc::ptr_eq(&road.point_b, &cities[0]))
            )
        });

        if !has_valid_roads {
            panic!("Invalid bus route: Not all consecutive stops in the route have existing roads between them.");
        }
    }

    fn add_event(&mut self, event: Arc<Event>, time: u32) {
        let bus_id = event.bus.get_id();
        self.event_queue.entry(time).or_insert_with(BTreeMap::new).insert(bus_id, event);
    }

    pub fn new_bus(&mut self, route: &[&Arc<City>]) {
        let route = route.iter().map(|&city| city.clone()).collect();
        self.valid_route(&route);
        let bus = Arc::new(Bus::new(route, self.next_bus_id));
        self.buses.push(bus.clone());
        self.next_bus_id += 1;
        let first_event = Event {
            bus: bus.clone(),
            city: bus.current_stop(),
            got_off_count: 0,
            got_on_count: 0,
        };
        self.add_event(Arc::new(first_event), self.current_time);
    }

    pub fn add_people(&mut self, from: &Arc<City>, to: &Arc<City>, count: u32) {
        // Retrieve or insert a new inner hashmap for the 'from' city
        let destination_counts = self.waiting_people.entry(from.clone()).or_insert_with(HashMap::new);

        // Add the number of people to the count for the destination city
        // If the destination city is not already in the map, it's inserted with the count
        *destination_counts.entry(to.clone()).or_insert(0) += count;
    }

    fn process_waiting_people(&mut self, event: Arc<Event>, current_time: u32) -> Arc<Event> {
        let destinations = self.waiting_people.get(&event.city).cloned();
        let mut event = Arc::try_unwrap(event).unwrap_or_else(|e| (*e).clone()); // Try to unwrap Arc, or clone the content

        if let Some(destinations) = destinations {
            for (destination, people_waiting) in destinations.iter() {
                if *people_waiting > 0 && event.bus.is_upcoming_stop(destination.clone()) {
                    let travel_time = event.bus.calculate_travel_time(&self.roads, destination.clone(), current_time);
                    
                    let mut bus_events = self.event_queue.entry(travel_time).or_insert_with(BTreeMap::new);
                    let existed_event = bus_events.entry(event.bus.get_id()).or_insert_with(|| 
                        Arc::new(Event {
                            bus: event.bus.clone(),
                            city: destination.clone(),
                            got_off_count: 0,
                            got_on_count: 0,
                        })
                    );

                    let mut existed_event = Arc::make_mut(existed_event);
                    existed_event.got_off_count += *people_waiting;
                    event.got_on_count += *people_waiting;
                    
                    // Reset the waiting count to 0
                    let city_waiting_people = self.waiting_people.get_mut(&event.city).unwrap();
                    *city_waiting_people.get_mut(destination).unwrap() = 0;
                }
            }
        }

        Arc::new(event)
    }

    pub fn execute(&mut self, time_units_count: u32) -> Vec<Arc<Event>> {
        let mut events = Vec::new();
        let end_time = self.current_time + time_units_count; // Calculate end time once

        for current_time in self.current_time..end_time {
            if let Some(bus_events) = self.event_queue.get_mut(&current_time) {
                let cloned_events: Vec<_> = bus_events.values().cloned().collect(); // Clone the bus events
                
                for event in cloned_events {
                    let processed_event = self.process_waiting_people(event, current_time);
                    processed_event.bus.move_to_next();
                    //if current_time == end_time - 1 {
                        events.push(processed_event);
                    //}
                }
            }
        }

        self.current_time += time_units_count; // Update the current time of the simulation

        events
    }
    
}

fn main() {
    println!("Hello, world!");
    let mut simulation = Simulation::new();
    let pls = simulation.new_city("Plzen");
    let prg = simulation.new_city("Prague");
    let brn = simulation.new_city("Brno");
    let ust = simulation.new_city("Usti");
    let d1 = simulation.new_road(&pls, &prg, 90);
    let d2 = simulation.new_road(&prg, &brn, 120);
    let d3 = simulation.new_road(&prg, &ust, 80);
    let d4 = simulation.new_road(&pls, &ust, 110);
    simulation.new_bus(&[&pls, &prg, &brn]);
    simulation.new_bus(&[&prg, &pls, &ust]);
    simulation.add_people(&prg, &brn, 50);
    simulation.add_people(&prg, &ust, 50);
    simulation.add_people(&pls, &ust, 50);
    simulation.add_people(&pls, &prg, 10);
    //simulation.add_people(&brn, &prg, 50);
    //simulation.test_calc(brn.clone());
    //simulation.test_calc(prg.clone());
    for event in simulation.execute(270) {
        let name = event.city().name();
        let people_got_off = event.got_off();
        let people_got_on = event.got_on();
        println!("At {}, {} people got off and {} people got on at {}", simulation.current_time, people_got_off, people_got_on, name);
    }
    for event in simulation.execute(90) {
        let name = event.city().name();
        let people_got_off = event.got_off();
        let people_got_on = event.got_on();
        println!("At {}, {} people got off and {} people got on at {}", simulation.current_time, people_got_off, people_got_on, name);
    }

}
