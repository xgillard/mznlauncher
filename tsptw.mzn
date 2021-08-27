%-------------------------------------------------------------------------------
% This model optimizes the travelling salesman with time windows for the 
% makespan objective. This is different from the published (slighly simpler) 
% model optimizing for the total travel distance.
%
% This model is based on the TSPTW model that was published in :
% ``Solving satisfaction problems using large-neighbourhood search''
% Bjordal et al, CP2020. 
%-------------------------------------------------------------------------------
% Usage:
% minizinc --output-time -p <num_cpu> -i tsptw.mzn <instance>.dzn
%-------------------------------------------------------------------------------
% Author: X. Gillard <xavier.gillard@uclouvain.be>
% Date:   August 27, 2021
%-------------------------------------------------------------------------------
include "globals.mzn";

%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% INPUT PARAMETERS
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% - n:            the number of cities involved in the tour
% - time_window:  an array 2d where each line time_window[i] is the tuple
%                 [earliest arrival time, latest arrival time].
% - distance:     the distance matrix between each city of the tour.
%                 distance[i, j] is the time it takes to travel from city i to
%                 city j.
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
int:                         n;
array[Cities,1..2]   of int: time_window;
array[Cities,Cities] of int: distance;

%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% USEFUL NOTATIONS
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% These notations immediately follow from the input parameters. They are 
% introduced for the sole purpose of easing the reading of this model 
% (and its writing !).
% 
% - depot:   this is the original starting point of the travelling salesman
% - Cities:  gives a clear type to the cities that must be visited
% - horizon: this is the latest time at which the tour must be completed.
%            (obviously, the latest latest arrival time specified in the 
%            timewindow)
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
int:        depot   = 1;
set of int: Cities  = 1..n;
int:        horizon = max(array1d(time_window));

%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% DECISION VARIABLES
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% What this problem really aims to do, is to find the best possible circuit
% going through all the cities while minimizing the makespan. The tour of cities
% is in the form of a chain of predecessors (pred) starting at the depot. 
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
array[Cities] of var Cities: pred;

%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% AUXILLIARY VARIABLES
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% - arrival_time[i] indicates the moment when the salesman *visits* city i.
%                   this includes the travel time + waiting time of the salesman
% - waiting_time[i] indicates the amount of time the salesman must wait before
%                   entering city[i]. 
%
%                   Note: keeping_track of the waiting time is mandatory when
%                   optimizing for makespan. Otherwise, the objective is on 
%                   travel time when only the travel distance alone is considered
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
array[Cities] of var 0..horizon: arrival_time;
array[Cities] of var 0..horizon: waiting_time;

%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% CONSTRAINTS
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% This establishes the general goal of finding a tour going through each city.
constraint circuit(pred)::domain;
% Enforces that the salesman must travel for at least as long as the time
% it takes to move from one city to the next
constraint forall(i in Cities where i != depot)(
  (arrival_time[i] >= arrival_time[pred[i]]+distance[pred[i],i])
);
%-------------------------------------------------------------------------------
% Enforces the satisfaction of the time windows for the visit of each city.
%-------------------------------------------------------------------------------
constraint arrival_time[depot] = time_window[depot,1];
constraint forall(i in Cities where i != depot)(
  (arrival_time[i] >= time_window[i,1])
);
constraint forall(i in Cities)(
  arrival_time[i] <= time_window[i,2]
);
%-------------------------------------------------------------------------------
% Enforce the correct tracking of the waiting time.
%-------------------------------------------------------------------------------
% **NOTE** 
% We enforce that waiting time be as small as possible as it rules out
% tons of symmetric solutions that are of no interest for us.
%-------------------------------------------------------------------------------
constraint waiting_time[depot] = 0;
constraint forall(i in Cities where i != depot)(
  waiting_time[i] = arrival_time[i] - arrival_time[pred[i]] - distance[pred[i], i]
);
% == enforce a minimal wait time before entering the city 
constraint forall(i in Cities where i != depot)(
  if (arrival_time[pred[i]] + distance[pred[i], i]) >= time_window[i, 1] then
    waiting_time[i] = 0
  else
    waiting_time[i] = time_window[i, 1] - (arrival_time[pred[i]] + distance[pred[i], i])
  endif
);

%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% OBJECTIVE FORMULATION
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% The overall objective of minimizing the makespan consists of minimizing the
% sum of the travel time and total wait time. This is made pretty explicit 
% in this next section.
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
var 0..horizon: total_travel_time;
var 0..horizon: total_wait_time;
var 0..horizon: makespan;

% == These line define the above partial and global objectives ==
constraint total_travel_time = sum(i in Cities)(distance[pred[i], i]);
constraint total_wait_time   = sum(i in Cities)(waiting_time[i]);
constraint makespan          = total_travel_time + total_wait_time;

% == In order to solve our TSPTW problem, it is sufficient to tell minizinc ==
% == `solve minimize makespan;` However, because we know that only `pred`   ==
% == are decision variables, it seems only fair to give it a good search    ==
% == strategy.                                                              ==
solve ::int_search(pred, dom_w_deg, indomain_random, complete) minimize makespan;

%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% OUTPUT SPECIFICATION.
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% This simply tells minizinc what to print when a solution is found. To that
% end, we simply compute the permutation of the solution and rebase it so that
% it is 0-indexed iso 1-indexed (which is used in minizinc).
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

% == Recursive function to tell the city which visited at ith step of the tour
function var int: ith(int: x) = if x == n then depot else pred[ith(x+1)] endif;

% == Utility function to rebase a city indetifier and make it 0-indexed
function var int: rebase(var int: x) = x -1;

% == Utility function which computes the permutation, aka the tour in an easy
% == to understand way
function array[Cities] of var int: permutation() = [rebase(ith(i)) | i in Cities ];

% == Simply stores the permutation so as to print it out
array[Cities] of var int: perm = permutation();

% == Actually print out any solution found
output[
  "% makespan: \(makespan/10000.0) \n" ++
  "% permutation: \(perm) \n"
]