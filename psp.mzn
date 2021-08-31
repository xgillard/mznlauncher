%-------------------------------------------------------------------------------
% This model optimizes the pigment sequencing problem.
%-------------------------------------------------------------------------------
% Usage:
% minizinc --output-time -p <num_cpu> -i psp.mzn <instance>.dzn
%-------------------------------------------------------------------------------
% Author: X. Gillard <xavier.gillard@uclouvain.be>
% Date:   August 30, 2021
%-------------------------------------------------------------------------------

%include "globals.mzn";
include "circuit.mzn";
include "maximum.mzn";
include "minimum.mzn";
include "sum_pred.mzn";
include "sum_set.mzn";

%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% INPUT PARAMETERS
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% - n:            the number of items that can be produced
% - horizon:      the number of time steps in the production plan.
% - changeover:   a matrix where changeover[i, j] is the price it costs to
%                 reconfigure the production machine from producing item i to 
%                 producing item j.
% - stocking:     stocking[i] is the price it costs to store one unit of item i
%                 for a period of one unit of time.
% - demands:      a vector2d st demands[i, t] tells whether or not n item of
%                 type i must be delivered at time t.      
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
int:                          n;
int:                          horizon;

array [Item, Item] of int:    changeover;
array [Item] of int:          stocking;
array [Item, Time] of int:    demands;

%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% USEFUL NOTATIONS
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% These notations immediately follow from the input parameters. They are 
% introduced for the sole purpose of easing the reading of this model 
% (and its writing !).
% 
% - Items:    gives a clear type to the items that must be created
% - Time:     gives a clear type to the timesteps of the planning
% - required: a bidimensional array such that required[i, t] denotes the total
%             quantity of items of type i that must have been delivered before
%             (or at) time t.
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
set of int:                   Item     = 1..n;
set of int:                   Time     = 1..horizon;

% function that computes the total demand for items of type i at time t
function int: i_demand(Item: i, Time: t) = 
    if t == 1 then demands[i, t] else demands[i, t] + i_demand(i, t-1) endif;

array [Item, Time] of int: requried = array2d(Item, Time,
    [ i_demand(i, t) | i in Item, t in Time ]
);
    

%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% DECISION VARIABLES
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% What this problem really aims to do, is to find the best possible production 
% plan which minimizes the total stocking and changeover cost.
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
array[Time] of var Item:      plan;

%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% AUXILLIARY VARIABLES
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% - prod[i,t]  tells true if item i was produced at time t. (this is mostly
%              useful to express the constraint on deadlines).
% - co_cost[i] gives the changeover cost incurred at time t. So co_cost[t] is
%              changeover[i, j] where i = prod[t] and j = prod[t+1]
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
array[Item, Time] of var bool: prod;
array[Time] of var int: co_cost;

%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% CONSTRAINTS
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% Enforces that prod[i, t] == true whenever item i is produced at time t.
constraint forall (i in Item) (
    forall (t in Time) (
        if plan[t] == i then prod[i, t] = 1 else prod[i, t] = 0 endif
    )
);
%-------------------------------------------------------------------------------
% Enforces the deadlines constraint. Any item can be produced at any time, as 
% long as the imposed delivery times are respected. (see demands)
%-------------------------------------------------------------------------------
constraint forall (i in Item) (
    forall (t in Time) (
        sum([prod[i, x] | x in 1..t]) >= requried[i, t]
    )
);
%-------------------------------------------------------------------------------
% Enforces that at all times, co_cost[t] is changeover[i, j]
% where i = prod[t] and j = prod[t+1]
%-------------------------------------------------------------------------------
constraint forall (t in Time)(
    if t < horizon /\ plan[t] != plan[t+1] then 
        co_cost[t] = changeover[plan[t], plan[t+1]] 
    else 
        co_cost[t] = 0 
    endif
);


%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% OBJECTIVE FORMULATION
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% The overall objective of minimizing the total cost consists of minimizing the
% sum of the changeover costs and the total stocking cost.
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% The total stocking cost
var int: tot_st_cost = sum([stocking[i] * (sum([prod[i, t2] | t2 in 1..t]) - requried[i, t]) | i in Item, t in Time]);
% The total changeover cost
var int: tot_co_cost = sum(co_cost);
% The overall objective that must be minimized
var int: total_cost  = tot_co_cost + tot_st_cost;

% == In order to solve our PSP problem, it is sufficient to tell minizinc   ==
% == `solve minimize total_cost;` However, because we know that only `plan` ==
% == are decision variables, it seems only fair to give it a good search    ==
% == strategy.                                                              ==
solve ::int_search(plan, dom_w_deg, indomain_random, complete) minimize total_cost;

%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% OUTPUT SPECIFICATION.
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
% This simply tells minizinc what to print when a solution is found. To that
% end, we simply compute the permutation of the solution and rebase it so that
% it is 0-indexed iso 1-indexed (which is used in minizinc).
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

% == Simply stores the solution so as to print it out                       ==
array[Time] of var int: solution = [ plan[t] -1 | t in Time ];


output [
    "% total cost : \(total_cost)  \n" ++
    "% stocking   : \(tot_st_cost) \n" ++ 
    "% changeover : \(tot_co_cost) \n" ++
    "% plan       : \(solution)"
]