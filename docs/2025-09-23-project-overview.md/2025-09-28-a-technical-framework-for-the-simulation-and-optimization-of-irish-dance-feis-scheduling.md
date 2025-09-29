

# **A Technical Framework for the Simulation and Optimization of Irish Dance Feis Scheduling**

## **Introduction**

An Irish dance feis (plural: *feiseanna*), Gaelic for "festival," is a vibrant celebration of culture, music, and dance, attracting competitors from numerous dance schools to a single venue.1 While appearing as a cultural gathering, from an operational perspective, a feis is a profoundly complex logistical event. A typical feis can range from a small local competition with around 400 dancers to a major regional event with over 1,000 participants, each competing in multiple dances across various skill levels and age groups throughout a single, long day.2 The smooth execution of such an event hinges on a single, critical element: the schedule.

The process of scheduling a feis is a formidable challenge, fraught with competing objectives and rigid constraints. Feis organizers, the hosts, are driven by the need for operational efficiency: maximizing the use of expensive resources like stages and certified judges, minimizing the total event duration (the "makespan") to control venue and personnel costs, and ensuring the entire program of competitions can be completed within the allotted time.4 Conversely, the attendees—dancers and their families—seek a positive and manageable experience. Their priorities include maximizing the dancer's ability to participate in their chosen events, minimizing excessive downtime between competitions, and avoiding the stress of a chaotic, unpredictable schedule where start times are merely suggestions.6

This inherent tension between host efficiency and attendee convenience places the feis scheduling problem squarely in the domain of multi-objective, resource-constrained combinatorial optimization. It shares characteristics with notoriously difficult problems in operations research, such as job-shop scheduling and university timetabling, but with its own unique set of rules, entities, and cultural nuances.9 The sheer number of variables—hundreds of dancers, dozens of competition categories, multiple stages, and a finite pool of judges—creates a combinatorial explosion of possible schedules, making manual or simple spreadsheet-based planning methods inadequate for achieving a truly optimal or even high-quality result.

This report presents a comprehensive technical framework for the development of a sophisticated software system designed to solve the Feis Scheduling Problem (FSP). The framework is composed of two primary, interconnected components. The first is a generative simulation engine capable of creating statistically realistic feis populations, modeling everything from dancer demographics and skill levels to their specific competition choices and registration patterns. This provides a robust and variable set of input data for testing and validation. The second component is a powerful scheduling solver. This report will formally define the FSP, detailing its variables, parameters, and a complex web of constraints. It will then explore and evaluate a range of algorithmic solutions, from exact methods like Constraint Programming to scalable metaheuristics, culminating in a proposal for a robust hybrid solver. Finally, the report will outline the system's architecture, defining the necessary user controls, the generation of actionable outputs like master and individual schedules, and a dashboard of Key Performance Indicators (KPIs) for quantitatively assessing the quality of any generated schedule. The objective is to provide a complete blueprint for an application that can transform the art of feis scheduling into a data-driven science, delivering significant value to both event organizers and the Irish dance community at large.

## **Section 1: Deconstruction of the Feis Ecosystem**

To computationally model and solve the Feis Scheduling Problem, it is first necessary to deconstruct the intricate ecosystem of an Irish dance competition into a structured set of quantifiable rules, entities, and parameters. This section provides a foundational analysis of the feis domain, translating its competitive structure, performance mechanics, and operational environment into a formal lexicon that can serve as the basis for the simulation and optimization models.

### **1.1 The Competitive Hierarchy: Dancer Levels, Age Groups, and Progression Pathways**

The structure of a feis is built upon a formal hierarchy that categorizes dancers to ensure fair competition. This hierarchy is defined by two primary axes: skill level and age group.

**Skill Levels:** Dancers progress through a sequence of seven primary skill levels, with advancement governed by performance in previous competitions. The typical progression is as follows 7:

1. **First Feis (or Pre-Beginner):** An introductory, non-graded level for dancers competing for the very first time, often limited to younger age groups.12  
2. **Beginner:** The first formal competitive level. Dancers typically remain in this level for their first year of competition.7 Some organizations subdivide this into Beginner 1 and Beginner 2\.13  
3. **Advanced Beginner:** For dancers with more than one year of experience who have not yet met the criteria to advance further.7  
4. **Novice:** Dancers who have achieved a top placement (typically 1st, 2nd, or 3rd) in Advanced Beginner.7  
5. **Prizewinner (or Open Prizewinner):** Dancers who have achieved a 1st place in Novice.7  
6. **Preliminary Championship (PC):** The first of the two championship tiers. Dancers qualify for this level after achieving 1st place in both a light shoe (Reel or Slip Jig) and a hard shoe (Treble Jig or Hornpipe) dance at the Prizewinner level.7  
7. **Open Championship (OC):** The highest level of competition. A dancer advances to OC after securing a set number of 1st place overall wins in Preliminary Championship, typically two or three depending on regional rules.7

In addition to these standard levels, special categories for adult dancers (often defined as over 18 or by other age brackets) exist, following a similar progression.11

**Age Groups:** Within each skill level, dancers are further subdivided into age groups to compete against their peers. A critical and universal rule is the concept of "feis age," which dictates that a dancer's competitive age for an entire calendar year is their age as of January 1st of that year.6 For example, a dancer who was 7 years old on January 1st will compete in the "Under 8" (U8) age group for the entire year, even after their 8th birthday. Competitions are thus organized by level and age, such as "U10 Novice" or "U16 Preliminary Championship."

**Progression Rules:** The mechanism for advancing through the Grade levels (Beginner to Prizewinner) is a core component of the system's logic and represents a significant hard constraint. Advancement is not based on a subjective assessment but on a strict, rule-based system. A dancer must achieve a specific placement (e.g., 1st, 2nd, or 3rd in Advanced Beginner) in a particular dance to move up to the next level *in that dance only*.7 This means a dancer can simultaneously be at the Novice level for the Reel but still at the Advanced Beginner level for the Hornpipe.

This system is further complicated by a crucial condition: for a placement to be considered valid for advancement, the competition must have a minimum number of participants. The most commonly cited threshold across various regions and organizations is **five or more dancers**.8 A 1st place win in a competition with only four dancers, while rewarding, does not count toward moving to the next skill level. This rule is not merely a procedural detail; it fundamentally shapes competitor behavior and introduces a strategic layer to the registration process. Dancers and their teachers monitor registration numbers on platforms like FeisWorx, and may strategically enter or withdraw from competitions to ensure their efforts are directed towards events that meet the advancement threshold. Any simulation or scheduling system must treat this "5-dancer rule" as a primary driver of competition formation and participant behavior.

### **1.2 The Repertoire of Competition: A Taxonomy of Soft Shoe, Hard Shoe, and Set Dances**

The dances performed at a feis are categorized primarily by the type of shoes worn, which produce distinctly different sounds and styles of movement. This distinction is fundamental to scheduling, as it necessitates time for dancers to change footwear between certain events.

**Shoe Types:**

* **Soft Shoes:** For female dancers, these are black leather shoes called "ghillies," which are similar to ballet slippers but with a more supportive structure.26 For male dancers, they are called "reel shoes," resembling black jazz shoes.27 These shoes are designed for dances that emphasize grace, height, and intricate footwork without percussive sound.  
* **Hard Shoes:** Also known as "jig shoes" or "heavy shoes," these are built with fiberglass or resin tips and heels that produce a loud, rhythmic, percussive sound when striking the stage.26 They are the basis for the powerful, rhythmic style of dance popularized by shows like  
  *Riverdance*.

**Dance Categories:**

* **Soft Shoe Dances:** This category includes the **Reel** (danced to music in 4/4 time), the **Light Jig** (6/8 time), the **Slip Jig** (9/8 time, traditionally performed only by female dancers), and the **Single Jig** (also known as Hop Jig, in 6/8 or 12/8 time).2  
* **Hard Shoe Dances:** This category is defined by its percussive footwork and includes the **Treble Jig** (or Heavy Jig, danced to 6/8 time music), the **Hornpipe** (2/4 or 4/4 time), and the **Treble Reel** (danced to reel music but with hard shoes).2 For Treble Jigs and Hornpipes at the Novice level and above, dancers may have the choice between "traditional" (fast) speed and "Oireachtas" (slow) speed music, with the slower tempo allowing for more complex, rhythmically dense steps.19  
* **Set Dances:** These are hard shoe dances performed to a specific, named piece of music. They are a required component of championship-level competitions. They are divided into two types:  
  * **Traditional Set Dances:** These are a specific group of dances with choreography that is fixed and known worldwide (e.g., St. Patrick's Day, Blackbird, Job of Journeywork).1  
  * **Non-Traditional (or Contemporary) Set Dances:** These are danced to a specific piece of music, but the choreography is uniquely created by the dancer's teacher, showcasing originality and technical difficulty. These are a hallmark of the Open Championship level.1

The scheduling of a dancer's day often involves moving between soft and hard shoe dances. This transition is not instantaneous. It requires a non-trivial amount of time to physically change shoes, with hard shoes often requiring careful lacing and sometimes taping to enhance sound or fit.33 This "shoe change time" is a hidden but critical temporal constraint that must be added to the minimum buffer time between a soft shoe and a hard shoe competition, adding a layer of necessary realism to the scheduling model.

### **1.3 The Mechanics of Competition: On-Stage Procedures, Performance Durations, and Adjudication Requirements**

The duration and resource requirements of each competition are not arbitrary; they are determined by a standardized set of procedures and rules that dictate how dancers perform and how they are judged.

**On-Stage Format:** In solo competitions, dancers do not perform one by one. Instead, they are brought onto the stage in small groups, typically of two or three at a time, depending on the size of the stage.2 They perform their steps simultaneously to the same live musician. As each dancer represents their own school, they will be performing different choreography, and the judge must assess each individual's performance within this shared space.2 This parallel performance format is a key factor in the overall time efficiency of a feis.

**Performance Duration:** The length of a performance is precisely defined by the music. In Irish dance, a "step" consists of a right-foot part and a left-foot part, each typically danced to eight bars of music, making one full step 16 bars long.2

* **Grade Levels (Beginner-Prizewinner):** Dancers perform two full steps of their dance, for a total of 32 bars of music.2  
* **Championship Levels (PC/OC):** Dancers perform longer, more complex routines, typically consisting of three full steps (48 bars) for Reels and Treble Jigs, or two-and-a-half steps (40 bars) for Slip Jigs and Hornpipes.1

The actual clock time for a performance depends on the tempo of the music, measured in Beats Per Minute (BPM). Tempos are standardized and vary by dance and skill level. For example, a championship Reel is typically played at 113 BPM, while a fast Treble Jig is at 92 BPM and a slow (championship) Treble Jig is at 73 BPM.19 By combining the number of bars, the time signature, and the specified tempo, a precise average duration can be calculated for each competition type, forming a critical input for the scheduling algorithm.

**Adjudication Requirements:** The number of judges required for a competition is a primary resource constraint that the scheduler must satisfy. The requirements are strictly tied to the competition level:

* **Grade Level Competitions:** These events (First Feis through Prizewinner) and team dances are adjudicated by a **single judge**.18  
* **Championship Competitions:** Both Preliminary and Open Championship events must be evaluated by a panel of **at least three adjudicators** to ensure fairness and smooth out subjective variations in scoring.1 Major championships like a regional Oireachtas or the World Championships can use panels of five, seven, or even nine judges for solo rounds.39

This bifurcation in judging requirements means that the pool of available judges is a constrained resource that must be allocated carefully. Championship stages will consume at least three times the judging resources as Grade level stages, a factor that heavily influences the overall layout of the feis schedule.

Finally, to manage the flow of the day, competitions with a large number of entrants (a common threshold is 21 or more) are often split into two smaller, roughly equal competitions.22 This ensures that no single event becomes unmanageably long and that judges are not required to assess an excessive number of dancers at once. The scheduling system must be capable of automatically performing these splits based on final registration numbers.

**Table 1: Dance Competition Parameters**

| Dance Name | Shoe Type | Applicable Levels | Time Signature | Bars of Music | Tempo (BPM) Range | Calculated Duration per Group (seconds) | Dancers on Stage |
| :---- | :---- | :---- | :---- | :---- | :---- | :---- | :---- |
| Reel | Soft | Grades | 4/4 | 32 | 112-130 | 30-34 | 2-3 |
| Reel | Soft | Championship | 4/4 | 48 | 112-116 | 50-51 | 2-3 |
| Light Jig | Soft | Grades | 6/8 | 32 | 112-121 | 32-34 | 2-3 |
| Slip Jig | Soft | Grades | 9/8 | 32 | 112-130 | 41-48 | 2-3 |
| Slip Jig | Soft | Championship | 9/8 | 40 | 112-118 | 61-64 | 2-3 |
| Treble Jig | Hard | Grades (Fast) | 6/8 | 32 | 85-96 | 40-45 | 2-3 |
| Treble Jig | Hard | Championship (Slow) | 6/8 | 48 | 72-76 | 95-100 | 2-3 |
| Hornpipe | Hard | Grades (Fast) | 4/4 | 32 | 128-144 | 27-30 | 2-3 |
| Hornpipe | Hard | Championship (Slow) | 4/4 | 40 | 112-116 | 41-43 | 2-3 |
| Traditional Set | Hard | Grades/Championship | Varies | Varies | Varies | 45-75 | 1 |

*Note: Calculated Duration is an estimate based on average tempo and music structure. It represents the time for one group of dancers to perform. A full competition's duration is this value multiplied by the number of groups, plus transition time.*

### **1.4 The Feis Environment: Event Scale, Physical Resources, and Personnel Constraints**

The final layer of the feis ecosystem involves the physical and human resources that define the operational capacity of the event. These are the primary inputs that an organizer would provide to a scheduling system.

**Feis Scale:** The overall size of a feis is the primary determinant of its complexity. Based on total competitor numbers, feiseanna can be categorized into three general archetypes, which will inform the simulation model 2:

* **Small Feis:** Approximately 400 competitors. Often a local, one-day event.  
* **Medium Feis:** Between 400 and 800 competitors.  
* **Large Feis:** 800 to over 1,000 competitors. These are often major regional events, sometimes spanning two days, and may have caps on the number of entries in certain categories to maintain manageability.20

**Physical Resources (Stages):** The number of available stages is a critical constraint. A large feis may operate 7 to 9 stages simultaneously in a large venue like a hotel ballroom or convention center.3 Each stage is an independent resource that can run one competition at a time. The number of stages is a key user-configurable input for the scheduling solver.

**Personnel Resources (Judges and Musicians):** Judges (adjudicators) and musicians are the essential, and often limited, human resources of a feis. An organizer will have a finite pool of certified judges and experienced musicians available for the day. The scheduling algorithm must not only assign competitions to stages and time slots but also assign the required number of judges and one musician to each stage for the duration of those competitions. The total number of available judges and musicians are therefore fundamental inputs to the system, directly constraining the number of stages that can be run concurrently, especially considering the higher demand from championship events.

## **Section 2: A Generative Model for Feis Competitor Simulation**

To develop and validate a robust scheduling solver, it is essential to have access to high-quality, realistic input data. Since real-world feis registration data is proprietary and varies widely, the first component of the proposed framework is a generative simulation engine. This engine is designed to create synthetic but statistically plausible populations of feis competitors and their event entries. This section details the architecture of this simulation model, providing a methodology for generating the datasets that will serve as the primary input for the scheduling solver.

### **2.1 Defining Feis Archetypes: Parameterizing Small, Medium, and Large Events**

The simulation begins with the user selecting a feis archetype, which pre-loads a set of parameters reflecting the typical characteristics of different-sized events. These archetypes are based on established competitor counts and provide a realistic starting point for the simulation.2

* **Small Feis (e.g., 400 dancers):** Typically a local event, characterized by a higher proportion of dancers in the lower Grade levels (First Feis, Beginner, Advanced Beginner). The number of Championship-level competitors is likely to be small.  
* **Medium Feis (e.g., 600 dancers):** A balanced event with a healthy mix of all skill levels, drawing from a wider geographic area.  
* **Large Feis (e.g., 1000 dancers):** Often a major regional event that attracts highly competitive dancers. This archetype will feature a significantly higher percentage of Preliminary and Open Championship competitors, leading to a greater demand for multi-judge panels.

The user can select one of these archetypes to generate a default population or choose a "Custom" option to specify the total number of dancers and manually adjust the distribution parameters, allowing for the simulation of unique or specific event scenarios.

### **2.2 A Stochastic Model for Dancer Generation: Simulating Demographics and Skill Distribution**

The core of the simulation is the generation of a population of individual Dancer objects. Each dancer is defined by a set of attributes that determine their eligibility for competitions. The generation process uses stochastic sampling from defined probability distributions to create a diverse and realistic cohort.

**The Dancer Object:** Each simulated dancer will have the following attributes:

* DancerID: A unique identifier.  
* Name: A randomly generated name for personalization.  
* Age: An integer representing the dancer's actual age.  
* FeisAge: The dancer's competition age, calculated as their age on January 1st of the simulated year.6  
* SkillLevel: The dancer's primary skill level (e.g., Novice, Preliminary Championship).

**Generation Process:**

1. **Population Size:** The total number of dancers, N, is determined by the selected feis archetype or custom input.  
2. **Age Distribution:** The age for each of the N dancers is sampled from a probability distribution. A log-normal distribution is a suitable choice, as it can be parameterized to reflect a population heavily skewed towards younger children and teenagers (e.g., ages 7-16), with a smaller, secondary mode or a long tail to represent the adult competitor population.  
3. **Skill Level Distribution:** A dancer's skill level is strongly correlated with their age and experience. The model will use a conditional probability table to assign a skill level based on the generated age. For example, the probability of an 8-year-old being a "Beginner" is very high, while the probability of them being an "Open Champion" is virtually zero. Conversely, a 17-year-old has a much higher probability of being in a Championship level. This ensures the generated population is logical and reflects the natural progression of a dancer's career.

**Table 2: Dancer Simulation Model Parameters**

| Parameter | Description | Data Type | Example Value (Medium Feis) |
| :---- | :---- | :---- | :---- |
| Total\_Dancers | The total number of competitors to simulate. | Integer | 600 |
| Age\_Distribution\_Type | The statistical distribution used to generate dancer ages. | String | "Lognormal" |
| Age\_Distribution\_Params | Parameters for the age distribution (e.g., mean and std. dev.). | Tuple | (μ=2.2, σ=0.4) |
| Skill\_Level\_Prob\_Table | A conditional probability table mapping age ranges to skill levels. | Dictionary | {'Ages\_5\_8': {'Beginner': 0.8,...}} |
| Avg\_Dances\_Per\_Grade\_Dancer | The average number of dances a Grade-level dancer enters. | Float | 3.5 |
| Dances\_Per\_Dancer\_Dist | The distribution for the number of dances entered per dancer. | String | "Poisson" |
| Prob\_Champ\_Dancer\_Attends | The probability that a generated Championship-level dancer will register. | Float | 0.95 |
| Shoe\_Change\_Time\_seconds | The fixed time penalty for a soft-to-hard shoe transition. | Integer | 180 |
| Inter\_Comp\_Buffer\_seconds | The minimum required time between any two of a dancer's competitions. | Integer | 300 |
| Strategic\_Registration\_Factor | A factor (0 to 1\) controlling the influence of the "5-Dancer Rule" on sign-ups. | Float | 0.7 |

### **2.3 Modeling Dancer Ambition: A Probabilistic Framework for Competition Selection**

Once the population of dancers is generated, the simulation must model their choices of which specific competitions to enter. This process is nuanced and depends heavily on the dancer's skill level, which serves as a strong proxy for their commitment and competitive goals.

For Grade-Level Dancers (First Feis to Prizewinner):  
These dancers face a menu of individual dance competitions for which they are eligible based on their level.13 The simulation models their selection process as follows:

1. **Determine Number of Dances:** For each Grade-level dancer, the total number of dances they will enter is sampled from a discrete probability distribution, such as a Poisson distribution. The mean of this distribution (Avg\_Dances\_Per\_Grade\_Dancer) is a key parameter from the feis archetype, reflecting that some dancers may enter only one or two dances, while highly ambitious ones might enter five or six.34  
2. **Select Specific Dances:** From the list of all eligible dances, the system randomly selects the determined number of dances. This selection can be weighted. For example, a dancer is more likely to enter the core soft shoe dances (Reel, Light Jig) before adding a hard shoe dance like the Hornpipe. The probability of selecting hard shoe dances increases as the dancer's skill level progresses from Advanced Beginner to Novice and Prizewinner.

For Championship-Level Dancers (PC and OC):  
The selection process for these elite dancers is fundamentally different. Their participation is not about choosing from a menu of dances but about deciding whether to enter the single, multi-round championship competition offered for their age group.1

1. **Determine Participation:** For each PC or OC dancer, the simulation uses a simple Bernoulli trial (a coin flip) to decide if they will compete. The probability of participation (Prob\_Champ\_Dancer\_Attends) is set to be very high (e.g., 95%), reflecting the high level of commitment required to reach this stage.  
2. **Assign Competition:** If the dancer is participating, they are entered into exactly one event: the Preliminary or Open Championship for their specific age group. This single entry automatically implies they will participate in all required rounds (e.g., a soft shoe round, a hard shoe round, and a set dance round for OC).

This bifurcated approach accurately models the different competitive structures within a single feis. It creates a dataset where the scheduling algorithm will encounter a large number of Grade-level dancers with relatively sparse, independent schedules, alongside a smaller but highly-constrained group of Championship dancers whose multi-round competitions create dense, interdependent scheduling requirements.

### **2.4 The Dynamics of Registration: Simulating Sign-up Velocity and Final Attendee Rosters**

To add a final layer of realism, the simulation does not generate the complete registration list instantaneously. Instead, it models the sign-up process over a period of several weeks leading up to the feis deadline, as this is how real-world registration occurs via platforms like QuickFeis and FeisWorx.6

**Registration Velocity:** The simulation uses a temporal model, such as an S-curve or a beta distribution, to model the rate of sign-ups over a 4-6 week registration window. This typically involves a slow start, an acceleration in the middle period, and a final surge of registrations just before the deadline.

**Strategic Registration Behavior:** This temporal model is essential for implementing the "strategic registration" behavior driven by the "5-Dancer Rule." As the simulation progresses through the registration period, the model for a dancer's competition selection (from Section 2.3) can be dynamically influenced by the current number of entrants in each competition. If the Strategic\_Registration\_Factor is high, a simulated dancer will be significantly less likely to register for a competition that has, for example, only two entrants, and more likely to choose an alternative dance that is closer to or has already surpassed the five-dancer threshold. This dynamic process results in a more realistic final distribution of competitors across events, reflecting the self-organizing nature of the feis community. The final output of the simulation is a complete and realistic roster of dancers and their confirmed competition entries, ready to be fed into the scheduling solver.

## **Section 3: The Feis Scheduling Problem (FSP): A Formal Model**

With a comprehensive understanding of the feis ecosystem and a method for generating realistic input data, the next critical step is to translate the scheduling challenge into a formal, mathematical model. This process of formalization is essential for applying rigorous, computer-based optimization techniques. It involves defining the problem's core entities, parameters, and decision variables, and articulating the complex web of rules and objectives as a precise set of mathematical constraints and a multi-objective function. This section provides that formal definition of the Feis Scheduling Problem (FSP).

### **3.1 Formal Problem Definition: Sets, Variables, and Parameters**

The FSP can be defined by the following components:

**Sets:** These represent the fundamental entities in the problem.

* D: The set of all registered dancers, d∈D.  
* C: The set of all competitions to be scheduled, c∈C. Each competition is a unique combination of a dance, a skill level, and an age group (e.g., "U10 Novice Reel").  
* S: The set of all available stages, s∈S.  
* J: The set of all available adjudicators (judges), j∈J.  
* M: The set of all available musicians, m∈M.  
* T: The set of discrete time slots representing the feis day, e.g., T={0,1,...,1440} for one-minute intervals from the start of the day.

**Parameters:** These are the known, fixed inputs to the problem, derived from the feis rules and the simulated registration data.

* duration(c): The total time required to run competition c, calculated from the number of entrants, dancers on stage at a time, and the per-group performance duration from Table 1\.  
* judges\_req(c): The number of judges required for competition c (1 for Grades, ≥3 for Championships).  
* dancers(c): The set of dancers d∈D registered for competition c.  
* shoe\_type(c): The shoe type (Soft or Hard) required for competition c.  
* rounds(c): For championship competitions, an ordered list of sub-competitions (rounds) that must be scheduled in sequence, e.g., (creel​,cjig​,cset​).  
* Tbuffer​: The minimum buffer time required between any two competitions for the same dancer.  
* Tshoe\_change​: The additional buffer time required if a dancer is switching between soft and hard shoes.

**Decision Variables:** These are the outputs the solver must determine.

* start\_timec​: An integer variable representing the start time slot t∈T for competition c.  
* stage\_asgnc​: A variable indicating which stage s∈S is assigned to competition c.  
* judge\_asgnc,j​: A binary variable that is 1 if judge j is assigned to competition c, and 0 otherwise.  
* musician\_asgnc,m​: A binary variable that is 1 if musician m is assigned to competition c, and 0 otherwise.

### **3.2 A Lexicon of Constraints: Delineating Hard and Soft Rules for a Feasible Schedule**

A valid schedule must satisfy a set of "hard" constraints, which are inviolable rules of the feis. "Soft" constraints represent desirable qualities of a schedule and are typically incorporated into the objective function.

**Hard Constraints:**

1. Dancer Uniqueness (No-Conflict): A dancer cannot participate in two competitions simultaneously. For any dancer d who is in two competitions c1​ and c2​, their time intervals cannot overlap, including the required buffer.  
   $$ \\forall d \\in D, \\forall c\_1, c\_2 \\in C \\text{ s.t. } d \\in dancers(c\_1) \\cap dancers(c\_2), c\_1 \\neq c\_2: $$ $$ (start\_time\_{c\_1} \+ duration(c\_1) \+ B(c\_1, c\_2) \\le start\_time\_{c\_2}) \\lor (start\_time\_{c\_2} \+ duration(c\_2) \+ B(c\_1, c\_2) \\le start\_time\_{c\_1}) $$  
   where B(c1​,c2​) is the buffer time, which equals Tbuffer​+Tshoe\_change​ if shoe\_type(c1​)=shoe\_type(c2​), and Tbuffer​ otherwise.  
2. Stage Uniqueness: A stage can only host one competition at a time.

   ∀c1​,c2​∈C s.t. stage\_asgnc1​​=stage\_asgnc2​​,c1​=c2​:  
   $$ (start\_time\_{c\_1} \+ duration(c\_1) \\le start\_time\_{c\_2}) \\lor (start\_time\_{c\_2} \+ duration(c\_2) \\le start\_time\_{c\_1}) $$  
3. Judge Uniqueness: A judge cannot be assigned to two overlapping competitions.  
   $$ \\forall j \\in J, \\forall c\_1, c\_2 \\in C, c\_1 \\neq c\_2 \\text{ where } judge\_asgn\_{c\_1,j}=1 \\text{ and } judge\_asgn\_{c\_2,j}=1: $$  
   The time intervals for c1​ and c2​ cannot overlap.  
4. Judge Requirement: Each competition must be assigned its required number of judges.

   ∀c∈C:j∈J∑​judge\_asgnc,j​=judges\_req(c)  
5. Musician Requirement and Uniqueness: Each competition must be assigned exactly one musician, and a musician cannot play for two overlapping competitions.

   ∀c∈C:m∈M∑​musician\_asgnc,m​=1

   (A uniqueness constraint similar to the judge constraint also applies).  
6. Precedence for Championship Rounds: For any championship competition c with ordered rounds (cr1​,cr2​,...), the rounds must be scheduled in sequence.  
   $$ \\forall \\text{round } i \\text{ in } rounds(c): start\_time\_{c\_{ri}} \+ duration(c\_{ri}) \\le start\_time\_{c\_{r(i+1)}} $$

The problem structure reveals itself as a hybrid of well-known optimization problems. The championship competitions, with their fixed sequence of rounds, resemble a Job-Shop Scheduling Problem (JSSP), where a "job" (the championship) has a series of "operations" (the rounds) that must be processed in order.9 In contrast, the Grade-level competitions for a given dancer are a set of independent tasks with no inherent order, which is more characteristic of a University Timetabling Problem (UTTP), where courses for a student must be scheduled without conflict.10 The FSP is therefore a complex amalgamation of both, requiring a flexible modeling approach that can handle both fixed precedence chains and unordered sets of tasks for the same shared resources (dancers, stages, judges). This hybrid nature makes declarative approaches like Constraint Programming, which excel at representing arbitrary and heterogeneous constraints, particularly well-suited for modeling the FSP.47

### **3.3 The Multi-Objective Function: Quantifying and Balancing Host Efficiency with the Dancer Experience**

The goal of the solver is not just to find *any* feasible schedule, but to find a *good* one. The quality of a schedule is measured by an objective function that captures the competing priorities of the host and the attendees. This is formulated as a weighted sum of several distinct objectives, where the weights allow the feis organizer to specify their priorities.

The function to be minimized is:

Minimize Z=w1​⋅fmakespan​+w2​⋅fidle​+w3​⋅fwait​  
Where:

* w1​,w2​,w3​ are user-defined weights such that ∑wi​=1.  
* fmakespan​, fidle​, and fwait​ are normalized scores for each objective.

Objective 1: Minimize Makespan (Host Priority)  
This objective aims to finish the entire feis as early as possible, reducing venue and staff costs.

fmakespan​=c∈Cmax​(start\_timec​+duration(c))  
Objective 2: Minimize Resource Idle Time (Host Priority)  
This objective promotes the efficient use of stages, judges, and musicians by minimizing the total time they are available but not in use.  
$$ f\_{idle} \= \\sum\_{s \\in S} \\text{IdleTime}(s) \+ \\sum\_{j \\in J} \\text{IdleTime}(j) \+ \\sum\_{m \\in M} \\text{IdleTime}(m) $$  
Where IdleTime(resource) is the total time within the feis makespan that the resource is not assigned to a competition.  
Objective 3: Minimize Dancer Wait Time & Dispersal (Dancer Priority)  
This is a more complex objective designed to improve the attendee experience by making their personal schedules more compact. It penalizes long gaps between a dancer's competitions.  
$$ f\_{wait} \= \\sum\_{d \\in D} \\left( (\\max\_{c \\in C\_d} (start\_time\_c \+ duration(c))) \- (\\min\_{c \\in C\_d} (start\_time\_c)) \- (\\sum\_{c \\in C\_d} duration(c)) \\right) $$  
Where Cd​ is the set of competitions entered by dancer d. This formula calculates, for each dancer, their total time on-site (from the start of their first event to the end of their last) and subtracts the time they are actually performing. The sum of this "net wait time" over all dancers is what the objective seeks to minimize.  
By adjusting the weights (w1​,w2​,w3​), a feis organizer can explore the trade-off space. A high weight on w1​ (makespan) might produce a very short feis day but could result in dancers having large, inconvenient gaps in their schedules. Conversely, a high weight on w3​ (wait time) would create highly compact schedules for individual dancers but would likely lead to a longer overall feis day with lower resource utilization. The ability to navigate this trade-off is a core feature of the proposed system.

## **Section 4: Algorithmic Solutions for the Feis Scheduling Problem**

Having formally defined the Feis Scheduling Problem (FSP), this section addresses the core technical challenge: the design and implementation of an algorithm capable of solving it. Given the FSP's NP-hard nature—a consequence of its combinatorial complexity—there is no one-size-fits-all solution. The choice of algorithm involves a trade-off between solution quality (optimality), computational time (scalability), and implementation complexity. This section explores a spectrum of algorithmic approaches, from exact methods that guarantee optimality for smaller problems to heuristic and hybrid strategies designed for the scale and dynamism of large feiseanna.

### **4.1 An Exact Method: A Constraint Programming (CP) Formulation for Optimal Solutions**

Constraint Programming (CP) is a powerful paradigm for solving combinatorial optimization problems. Unlike traditional programming, CP is declarative: the user specifies the variables, their domains, and the constraints (relations) that must hold between them, and the CP solver uses a combination of search and logical inference (constraint propagation) to find a solution.10

**Applicability to the FSP:** CP is exceptionally well-suited to the FSP for several reasons. Its declarative nature allows for a direct and intuitive translation of the complex and often heterogeneous rules of a feis (as detailed in Section 1\) into a formal model. Global constraints, which are high-level constraints encapsulating common combinatorial patterns, can elegantly model many of the FSP's requirements.48

* The **uniqueness constraints** (a dancer, stage, or judge can only be in one place at a time) can be modeled using interval variables and a noOverlap constraint. An interval variable is a specialized variable that represents an activity with a start time, end time, and duration, making it perfect for representing a competition.  
* The **resource requirement constraints** (e.g., a championship needs three judges) can be modeled using cumulative or cardinality constraints.  
* The **precedence constraints** for championship rounds can be modeled with simple temporal logic: endOf(round\_1) \<= startOf(round\_2).

**Implementation and Limitations:** A CP model of the FSP could be implemented using modern solver libraries like Google OR-Tools, CP-SAT, or ILOG CP Optimizer. For small- to medium-sized feiseanna, a CP solver may be able to find a provably optimal solution with respect to the defined objective function. However, the search space for the FSP grows exponentially with the number of competitions and resources. For a large feis with over 1,000 dancers and hundreds of distinct competitions, the computational time required to find and prove an optimal solution can become prohibitively long, potentially taking many hours or even days.47 This limitation necessitates the exploration of faster, albeit non-optimal, methods.

### **4.2 Heuristic and Metaheuristic Approaches: Strategies for Large-Scale Feiseanna**

When exact methods are too slow, heuristic and metaheuristic algorithms provide a practical alternative. These methods do not guarantee optimality but are designed to find very high-quality solutions in a reasonable amount of time.

**Priority Dispatch Rules (Heuristics):** These are simple, greedy rules used to construct a schedule one competition at a time. They are extremely fast but often yield suboptimal results. Examples tailored to the FSP include:

* **Earliest Start Time:** Always schedule the next available competition that can start the earliest.  
* **Most Constrained First:** Prioritize scheduling competitions that are the most difficult to place, such as Open Championship events, which require three judges and have multiple rounds with precedence constraints.  
* **Shortest Processing Time (SPT):** Schedule competitions with the shortest duration first. This is a classic rule from job-shop scheduling known to perform well for minimizing overall flow time.44

**Metaheuristic Approaches:** These are higher-level strategies that guide the search for a good solution by iteratively improving an existing schedule. They are well-suited for navigating the vast search space of the FSP.

* **Genetic Algorithms (GA):** In a GA approach, a "chromosome" would represent a complete schedule (an assignment of all competitions to times and stages). The algorithm would start with a population of random valid schedules. The "fitness" of each schedule is calculated using the multi-objective function from Section 3.3. The algorithm then proceeds through generations, using operators like "crossover" (combining two good schedules to create offspring) and "mutation" (making a small random change to a schedule) to evolve the population towards higher-fitness solutions.  
* **Simulated Annealing (SA) or Tabu Search:** These are local search algorithms. They start with an initial schedule and iteratively explore its "neighborhood" by making small changes (e.g., moving one competition to a different time slot, swapping the stages of two competitions). Changes that improve the objective function score are accepted. To avoid getting stuck in a local optimum, SA will occasionally accept a "bad" move with a probability that decreases over time, while Tabu Search maintains a list of recent moves that are temporarily forbidden.

### **4.3 A Proposed Hybrid Solver: Integrating Constraint Propagation with Local Search for Robustness and Scalability**

The most effective and practical solution for the FSP is likely a hybrid approach that combines the strengths of exact and heuristic methods. This approach, often called Large Neighborhood Search (LNS) or relaxation-based search, can produce high-quality solutions quickly and scale to large problem instances.

The proposed hybrid solver operates in two phases:

1. **Phase 1: Initial Feasible Solution Generation:** The process begins by using the Constraint Programming model from Section 4.1. However, instead of running it to optimality, it is run with a strict time limit (e.g., 60 seconds). The goal is not to find the *best* schedule, but to quickly find an initial *feasible* schedule that satisfies all the hard constraints. CP solvers are exceptionally good at this "satisfiability" task, leveraging powerful constraint propagation techniques to prune the search space of invalid assignments.  
2. **Phase 2: Iterative Improvement via Local Search:** Once a valid starting schedule is found, the algorithm switches to a metaheuristic local search. In each iteration, the algorithm will "relax" a small part of the schedule (e.g., by "un-scheduling" all competitions for a specific age group, or all events on a particular stage for a two-hour window). It then uses the CP solver again, but this time to optimally re-solve only this small, localized subproblem, while keeping the rest of the schedule fixed. Because the subproblem is much smaller, the CP solver can find an optimal solution for it very quickly. This process is repeated thousands of times, iteratively improving different parts of the schedule and gradually converging on a high-quality global solution.

This hybrid approach leverages CP for its logical rigor in handling complex constraints and local search for its speed and scalability in exploring a vast solution space, providing a robust and powerful engine for solving the FSP.

### **4.4 The Role of Large Language Models (LLMs): Applications in Heuristic Generation and Schedule Refinement**

While the core FSP is a structured optimization problem best solved by dedicated algorithms, Large Language Models (LLMs) can play a valuable supporting role in the overall system, particularly in areas requiring human-like reasoning and natural language interaction.

**Heuristic Discovery:** An LLM can be used as a "brainstorming partner" to generate scheduling strategies. An organizer could prompt an LLM with the rules of a feis and ask it to propose common-sense heuristics. For example, an LLM might suggest: "Group all the U8 and U9 competitions in the morning to allow families with young children to leave early," or "Try to run all Reel competitions on the same stage to minimize musician changes." These natural language suggestions can then be translated by system designers into new priority rules or soft constraints for the heuristic or CP solver, effectively using the LLM to augment the algorithmic search strategy.49

**Schedule Explanation and Interactive Refinement:** After the solver generates a schedule, it exists as a large dataset of assignments. An LLM could be used to translate this data into a human-readable summary. For instance, it could generate a report stating: "The final schedule has a total duration of 10.5 hours. To achieve this, some dancers in the U12 age group have a two-hour gap between their soft shoe and hard shoe dances. The stages are operating at 85% capacity." This provides valuable context to the organizer. Furthermore, this could be extended into an interactive loop. An organizer could provide feedback in natural language, such as, "Can we try to fix the long gap for the U12 dancers, even if it makes the day 15 minutes longer?" The LLM could help parse this request and translate it into a modified constraint or objective weight to be fed back into the solver for another run, creating a more intuitive and user-friendly optimization experience.

It is important to recognize that the scheduling process is not a one-time, pre-event calculation. The reality on the day of a feis is dynamic and often chaotic; competitions can run ahead of or behind schedule, and unforeseen issues can arise.6 This means a truly effective system must not only generate a high-quality initial plan but also be capable of adapting to real-time events. The proposed algorithmic solutions, particularly the faster heuristic and hybrid methods, are well-suited for this dynamic context. They can be re-run quickly from the current state of the feis to generate an updated schedule for the remainder of the day in response to significant delays, transforming the application from a static planning tool into a live operational management system.

## **Section 5: System Architecture, Outputs, and Performance Evaluation**

The translation of the proposed algorithmic solutions into a functional and valuable software application requires a well-defined system architecture, a clear set of user controls, actionable outputs, and a robust framework for performance evaluation. This section outlines the practical implementation of the feis scheduling system, detailing the interface for feis organizers, the reports and schedules it generates for all stakeholders, and a comprehensive dashboard of Key Performance Indicators (KPIs) to quantitatively measure the quality and effectiveness of any generated schedule.

### **5.1 User-Defined Inputs and Control Parameters**

The system's interface will provide the feis organizer (the user) with a set of controls to configure both the simulation and the optimization processes, ensuring the tool is adaptable to the unique needs of their specific event.

**Simulation Controls:**

* **Feis Archetype Selection:** The user can select a "Small," "Medium," or "Large" feis archetype to pre-load a standard set of simulation parameters, or choose "Custom" to manually define them.  
* **Dancer Population Parameters:** If using a custom setup, the user can adjust the core parameters outlined in Table 2, such as Total\_Dancers, the parameters for the age distribution, and the average number of dances per competitor. This allows for what-if analysis and capacity planning.

**Resource Controls:** These are the mandatory inputs for any scheduling run, defining the physical and human constraints of the event.

* Number of Stages: The total number of available performance stages.  
* Number of Judges: The total number of available adjudicators.  
* Number of Musicians: The total number of available musicians.  
* Feis Operating Hours: The start time (e.g., 8:00 AM) and a hard end time (e.g., 9:00 PM) for the event.

**Optimization Controls:** These controls allow the user to guide the solver's priorities.

* **Objective Function Weights:** A set of sliders or input fields for the user to adjust the weights (w1​,w2​,w3​) for the Makespan, Resource Idle Time, and Dancer Wait Time objectives. This allows them to explicitly manage the trade-off between operational efficiency and attendee convenience.  
* **Buffer Times:** The user can set the default values for Inter-Competition\_Buffer\_seconds and Shoe\_Change\_Time\_seconds to reflect the specific layout of their venue and their desired safety margin.

### **5.2 Generating Actionable Outputs: The Master Schedule, Individual Itineraries, and Resource Assignments**

Upon completion of a successful solver run, the system will generate a suite of outputs, each tailored to the needs of a specific stakeholder group. These outputs should be available in both human-readable formats (for on-screen viewing and printing) and data formats (like CSV or JSON) for integration with other systems.

* **Master Schedule (for Hosts):** This is the global, command-center view for the feis organizers. It would be best represented as a Gantt chart, with time on the x-axis and stages on the y-axis. Each colored block would represent a competition, displaying its name, number of entrants, and assigned judge(s). This provides a complete visual overview of the day's flow.  
* **Individual Itinerary (for Dancers/Parents):** For each dancer, the system will generate a personalized schedule. This is a critical output for improving the attendee experience. The itinerary will list, in chronological order:  
  * Competition Name (e.g., U10 Novice Reel)  
  * Competition Number  
  * Estimated Start Time  
  * Stage Number/Name  
  * Required Shoe Type  
    This output is ideally suited for a mobile app, which could provide push notifications to alert a dancer when their next competition is approaching (e.g., "Your competition, 405RL, is scheduled to begin in 20 minutes on Stage 3").51 This directly addresses the significant attendee pain point of schedule uncertainty.  
* **Resource Schedules (for Judges/Musicians):** Each judge and musician will receive a personal schedule detailing their assignments throughout the day, including stage assignments, competition blocks, and scheduled breaks. This facilitates professional management of event staff.

### **5.3 A Dashboard of Key Performance Indicators (KPIs) for Schedule Assessment**

To allow organizers to objectively evaluate the quality of a generated schedule and compare different scenarios, the system will present a dashboard of relevant KPIs. These metrics, inspired by best practices in general event management but tailored specifically to the feis context, quantify the schedule's performance against the core objectives.53

The most valuable contribution of a scheduling system is not just efficiency, but predictability. The primary source of stress and dissatisfaction for feis attendees is the inherent uncertainty of the day, where printed schedules are only approximations and stages run at their own pace.6 A schedule that appears mathematically optimal but has no built-in slack is operationally fragile and guaranteed to fail, causing cascading delays. Therefore, a successful system must produce schedules that are not only efficient but also robust. The KPIs must reflect this dual priority. A metric for "Schedule Robustness," such as the total amount of programmed buffer time or the average slack between consecutive competitions on a stage, is essential. This allows an organizer to see not just how efficient a schedule is, but how resilient it is to the inevitable minor delays of a live event.

**Table 3: Schedule Evaluation KPIs**

| Category | KPI Name | Unit | Description |
| :---- | :---- | :---- | :---- |
| **Host-Centric (Efficiency)** | Makespan (Total Duration) | Hours | The time from the start of the first competition to the end of the last. A primary measure of overall event length and cost. |
|  | Overall Stage Utilization | % | The total time stages are actively used for competition, divided by the total time they are available within the makespan. |
|  | Overall Judge Utilization | % | The total time judges are assigned to competitions, divided by the total time they are available. |
|  | Average Stage Idle Time | Minutes | The average time a stage sits empty between two consecutive competitions. |
|  | Total Competitions Scheduled | Count | The total number of individual competition events successfully placed on the schedule. |
| **Dancer-Centric (Experience)** | Average Dancer On-Site Time | Hours | The average duration from a dancer's first competition start time to their last competition end time. |
|  | Median Dancer On-Site Time | Hours | The median on-site time, which is more robust to outliers (e.g., a single dancer with events at the very beginning and end of the day). |
|  | Average Schedule Compactness | Index (0-1) | A normalized score representing how tightly clustered a dancer's events are. Calculated as (Total Performance Time) / (On-Site Time). Higher is better. |
|  | Max Inter-Competition Wait | Hours | The single longest wait time any dancer experiences between two of their competitions. Highlights the worst-case experience. |
|  | % of Dancers with \>2hr Gaps | % | The percentage of all dancers who have at least one gap of two hours or more in their personal schedule. A measure of widespread inconvenience. |
| **Operational (Robustness)** | Total Scheduled Slack Time | Minutes | The sum of all buffer time explicitly scheduled between competitions on all stages. A measure of the schedule's resilience to minor delays. |

This dashboard provides a multi-faceted view of the schedule's quality. It empowers the feis organizer to make informed decisions, understand the consequences of their resource allocation, and consciously balance the operational needs of the event with the goal of creating a positive, predictable, and enjoyable experience for the competitors.

## **Section 6: Real-Time Operations and Dynamic Schedule Repair**

The generation of a pre-event schedule, however optimal, is only the first step. The day of a feis is a dynamic environment where unforeseen events—such as competitions running long, dancers arriving late, or no-shows—are inevitable.58 A truly effective system must therefore transition from a

*predictive* scheduling tool to a *reactive* one, capable of handling real-time disruptions intelligently and with minimal turmoil. This requires a fundamental shift in both the algorithmic approach and the optimization objectives.

### **6.1 A New Objective: Minimizing Schedule Perturbation**

On the day of the event, the primary goal is no longer global optimization of makespan or dancer wait times. Instead, the new, overriding objective is **schedule stability**. When a disruption occurs, the best solution is the one that resolves the issue while causing the least amount of change to the existing, published schedule.61 Total rescheduling is a poor alternative as it creates confusion and stress for attendees who have already planned their day around the initial timeline.62

This new objective can be formalized as a "minimal perturbation" or "schedule repair" problem. The cost function to be minimized is not the overall quality of the schedule, but the magnitude of the deviation from the current schedule. This can be quantified in several human-relatable ways:

* **Minimize Number of Affected Dancers:** The primary goal is to find a repair that impacts the personal itineraries of the fewest possible individuals.  
* **Minimize Total Schedule Shift:** This measures the sum of all changes in start times across all remaining competitions. A solution that delays ten competitions by one minute is preferable to one that delays a single competition by fifteen minutes.  
* **Minimize High-Impact Changes:** This applies a heavier penalty to changes that are particularly disruptive, such as moving a competition to a different stage or creating a new schedule conflict for a dancer that was previously conflict-free.

### **6.2 A Human-in-the-Loop (HITL) Framework for Schedule Repair**

To manage day-of events effectively, the system must combine algorithmic speed with human judgment. A Human-in-the-Loop (HITL) approach is ideal for this, allowing on-site staff like stage managers to act as the "human" who initiates and validates schedule repairs.63 This creates a collaborative process between the human operator and the AI-powered scheduling engine.63

The workflow for a typical disruption (e.g., a dancer is a no-show for a competition) would be as follows:

1. **Event Trigger (Human Input):** A stage manager, using a tablet or mobile device, logs the event: "Dancer \#247 is a no-show for competition \#512HP."  
2. **Rapid Solution Generation (The Algorithm):** The system does not re-solve the entire feis schedule. Instead, it performs a very fast, localized "schedule repair".62 It calculates the immediate impact (e.g., competition \#512HP will now finish 90 seconds earlier) and generates a small set of ranked repair options in a matter of seconds. These options are generated by a specialized, lightweight algorithm—not the full CP solver—that is optimized for speed and minimal perturbation.  
3. **Presenting Human-Centric Options:** The system presents the stage manager with a few clear, understandable choices:  
   * **Option 1 (Absorb Delay):** "Create a 90-second gap before the next competition. *Impact: No other competitions or dancers are affected.*" This is the least disruptive option.  
   * **Option 2 (Local Pull-Forward):** "Pull all subsequent competitions on Stage 5 forward by 90 seconds. *Impact: Affects 47 dancers on this stage. No new conflicts created.*" This option prioritizes stage efficiency over schedule stability.  
   * **Option 3 (Swap):** "Swap the order of the next two competitions (\#513RL and \#514LJ). *Impact: Affects 12 dancers. Resolves a potential tight connection for Dancer \#315.*" This demonstrates a more complex, but potentially beneficial, local adjustment.  
4. **Human-Driven Decision and Execution:** The stage manager, using their on-the-ground knowledge and judgment, selects the best option.66 With a single tap, the system executes the change. The master schedule is instantly updated, and automated push notifications are sent  
   *only* to the affected dancers, informing them of their new estimated start time.67

### **6.3 Tiered Change Management and Approval**

Not all disruptions are equal. The system should be designed with tiered permissions to ensure that minor, routine adjustments can be handled efficiently at the local level, while major disruptions receive appropriate oversight.64

* **Level 1 (Ad-Hoc Adjustments):** These are minor events like a single no-show, a dancer needing to swap their position within a competition, or a competition running a few minutes ahead or behind schedule. These changes can be proposed by the system and approved directly by the designated stage manager without further authorization.  
* **Level 2 (Significant Disruptions):** These are larger issues, such as a stage running more than 30 minutes late, a judge becoming unavailable for an extended period, or a technical failure (e.g., sound system malfunction). In these cases, the system would still generate repair proposals, but these would require approval from a central feis administrator or director. The system would highlight the widespread impact of the proposed changes (e.g., "This solution will affect 150 dancers and extend the feis by 25 minutes") to inform the high-level decision.

This dynamic, human-in-the-loop repair capability transforms the system from a static planning tool into a live operational co-pilot. It empowers feis organizers to manage the inherent chaos of a live event with agility and intelligence, drastically reducing on-site turmoil and improving the experience for everyone involved.

## **Section 7: Conclusion and Strategic Recommendations**

The scheduling of an Irish dance feis is a formidable combinatorial optimization problem, characterized by a complex interplay of rigid rules, finite resources, and the conflicting objectives of event hosts and attendees. The traditional, manual approaches to this task are ill-equipped to navigate this complexity, often resulting in schedules that are inefficient, unpredictable, and stressful for participants. The technical framework detailed in this report provides a systematic, data-driven methodology for solving the Feis Scheduling Problem, transforming it from an art into a science.

The analysis has established that the FSP is a unique hybrid of classical scheduling problems, combining the fixed precedence constraints of job-shop scheduling for championship events with the independent task allocation of timetabling problems for grade-level competitions. This structural complexity, coupled with the sheer scale of a large feis, demands a sophisticated algorithmic approach. The proposed hybrid solver, which integrates the logical rigor of Constraint Programming for finding initial feasible solutions with the speed and scalability of metaheuristic local search for iterative optimization, represents the most robust and practical path forward. This approach is capable of generating high-quality schedules that explicitly and quantitatively balance the host's need for efficiency with the dancer's desire for a compact and predictable day.

Furthermore, the framework emphasizes that a successful system must be built upon a foundation of deep domain knowledge. The simulation engine is designed to generate realistic competitor populations by modeling nuanced behaviors, such as the strategic registration patterns driven by the "5-dancer rule" for advancement. The scheduling model incorporates critical, often overlooked, constraints like the additional time required for shoe changes between soft and hard shoe dances. The proposed Key Performance Indicators move beyond simple efficiency metrics to include measures of the dancer experience and, crucially, the schedule's operational robustness—its ability to absorb the minor delays inherent in any live event. The addition of a dynamic, real-time schedule repair module, operating on the principle of minimal perturbation, elevates the system from a pre-event planning tool to a live operational management system, capable of handling the inevitable disruptions of the feis day with agility and minimal impact on attendees.

Based on this comprehensive analysis, the following strategic recommendations are proposed for the development of a feis scheduling application:

1. **Adopt a Phased Implementation Roadmap:** The complexity of the system lends itself to a phased development approach to manage risk and deliver value incrementally.  
   * **Phase 1 (Minimum Viable Product):** Focus on building the core domain model (Section 1\) and implementing a simple, heuristic-based scheduler (e.g., using priority dispatch rules). This initial version would already provide more structure and automation than manual methods and would serve as a platform for gathering user feedback.  
   * **Phase 2 (Core Solver Implementation):** Develop the full hybrid solver (CP \+ Local Search) and the detailed competitor simulation engine. This phase delivers the core optimization capability, allowing organizers to generate and compare high-quality, robust schedules for simulated or real event data.  
   * **Phase 3 (Live Operational Management):** Extend the system to function as a real-time, day-of-event management tool, as detailed in Section 6\. This involves developing a mobile interface for attendees to view live schedule updates and for stage managers to report progress and execute schedule repairs. The core solver should be engineered to enable rapid re-scheduling from the current point in time, providing organizers with the intelligence to dynamically manage delays and disruptions.  
2. **Prioritize the User Experience for All Stakeholders:** The ultimate success of the application depends on its adoption by both organizers and attendees. The interface for organizers should be intuitive, clearly visualizing the trade-offs involved in their scheduling decisions. For dancers and parents, the output must be a clear, personalized, and dynamically updated itinerary, delivered through a mobile app with features like push notifications to dramatically reduce the uncertainty and stress that currently characterize the feis experience.  
3. **Embrace the Schedule as a Plan, Not a Mandate:** The system's design philosophy should acknowledge that any pre-generated schedule is a plan that will inevitably encounter the friction of reality. The greatest value the application can provide is not in generating a single, "perfect" static schedule, but in its ability to create a robust initial plan and then provide the real-time information and re-planning capabilities needed to intelligently manage the event as it unfolds.

By following this framework, it is possible to create a powerful tool that not only solves a complex technical problem but also delivers significant and tangible benefits to the entire Irish dance community, making feiseanna more efficient to run, more predictable to attend, and ultimately, more enjoyable for everyone involved.

#### **Works cited**

1. en.wikipedia.org, accessed September 22, 2025, [https://en.wikipedia.org/wiki/Feis](https://en.wikipedia.org/wiki/Feis)  
2. Feiseanna 101 \- Teelin Irish Dance, accessed September 22, 2025, [https://teelin.com/\_pdf/Feisanna%20101.pdf](https://teelin.com/_pdf/Feisanna%20101.pdf)  
3. FIRST FEIS INFO \- Paloma Irish Dance, accessed September 22, 2025, [https://www.palomairishdance.com/first-feis-info.html](https://www.palomairishdance.com/first-feis-info.html)  
4. FEIS 101 – How to Read a Feis Schedule \- Squarespace, accessed September 22, 2025, [https://static1.squarespace.com/static/5edb01e3d2329c6ac3435a23/t/60d36507c67c9405f7f9db12/1624466695476/How+to-+Feis+Schedule.pdf](https://static1.squarespace.com/static/5edb01e3d2329c6ac3435a23/t/60d36507c67c9405f7f9db12/1624466695476/How+to-+Feis+Schedule.pdf)  
5. MEET ORGANIZATION GUIDELINES \- USA Gymnastics, accessed September 22, 2025, [https://static.usagym.org/PDFs/Women/Rules/Rules%20and%20Policies/w-meetorgguide.pdf](https://static.usagym.org/PDFs/Women/Rules/Rules%20and%20Policies/w-meetorgguide.pdf)  
6. Feis FAQs What is a feis and other important information about about feiseanna — O'Hare Irish Dance, accessed September 22, 2025, [https://ohareirishdance.com/feis-faqs](https://ohareirishdance.com/feis-faqs)  
7. Feis Info | McDade Cara, accessed September 22, 2025, [https://mcdadecara.com/feis-info/](https://mcdadecara.com/feis-info/)  
8. Feis Information for Parents \- Keenan Irish Dance School, accessed September 22, 2025, [https://keenanirishdanceschool.com/wp-content/uploads/2011/01/feisinfoforparents.pdf](https://keenanirishdanceschool.com/wp-content/uploads/2011/01/feisinfoforparents.pdf)  
9. Job-shop scheduling \- Wikipedia, accessed September 22, 2025, [https://en.wikipedia.org/wiki/Job-shop\_scheduling](https://en.wikipedia.org/wiki/Job-shop_scheduling)  
10. Constraint-based Timetabling \- UniTime, accessed September 22, 2025, [https://www.unitime.org/papers/phd05.pdf](https://www.unitime.org/papers/phd05.pdf)  
11. mcdadecara.com, accessed September 22, 2025, [https://mcdadecara.com/feis-info/\#:\~:text=In%20Irish%20dance%2C%20there%20are,gaining%20medals%20in%20different%20dances.](https://mcdadecara.com/feis-info/#:~:text=In%20Irish%20dance%2C%20there%20are,gaining%20medals%20in%20different%20dances.)  
12. Competition Guide — ODonnell Academy of Irish Dance, accessed September 22, 2025, [https://www.newyorkirishdance.com/competition-guide](https://www.newyorkirishdance.com/competition-guide)  
13. Feis Levels \- O'Hare Irish Dance, accessed September 22, 2025, [https://ohareirishdance.com/feis-levels](https://ohareirishdance.com/feis-levels)  
14. Competition Levels | anclarschool, accessed September 22, 2025, [https://www.anclarschool.com/competition-levels](https://www.anclarschool.com/competition-levels)  
15. COMPETITION INFO \- Clarkson School of Irish Dance, accessed September 22, 2025, [https://www.clarksonschool.com/feis-info](https://www.clarksonschool.com/feis-info)  
16. Competition Levels \- Grades & Championship \- Slattery School of Irish Dance, accessed September 22, 2025, [http://www.slatteryirishdance.com/competition-levels.html](http://www.slatteryirishdance.com/competition-levels.html)  
17. Intro to Feising Information | Murphy Irish Dance, accessed September 22, 2025, [https://www.murphyacademy.com/feis-information](https://www.murphyacademy.com/feis-information)  
18. Feis FAQ \- Clarkson School of Irish Dance, accessed September 22, 2025, [https://www.clarksonschool.com/feis-faq](https://www.clarksonschool.com/feis-faq)  
19. Feis Ranks \- Teelin Irish Dance, accessed September 22, 2025, [https://teelin.com/\_pdf/Feis%20Ranks.pdf](https://teelin.com/_pdf/Feis%20Ranks.pdf)  
20. QuickFeis Syllabus \- Planxti, accessed September 22, 2025, [https://planxti.com/files/feis/file/943188c3-526e-4cdc-ae57-0edf500f71a4-8c9f9c76-5168-45f3-81b5-4457cc9eb9fa-2023-08-16T01-48:37-289Z.pdf](https://planxti.com/files/feis/file/943188c3-526e-4cdc-ae57-0edf500f71a4-8c9f9c76-5168-45f3-81b5-4457cc9eb9fa-2023-08-16T01-48:37-289Z.pdf)  
21. Feis Schedule – Teelin School of Irish Dance, accessed September 22, 2025, [https://teelin.com/TSID/about-competitions/feis-schedule/](https://teelin.com/TSID/about-competitions/feis-schedule/)  
22. Competition Rules – World Irish Dance Association, accessed September 22, 2025, [https://www.irish.dance/competition-rules](https://www.irish.dance/competition-rules)  
23. Competitive Irish Dance \- Rise Academy of Dance, accessed September 22, 2025, [https://riseacademydance.com/competitive-irish-dance](https://riseacademydance.com/competitive-irish-dance)  
24. Competitions \- Culkin School of Traditional Irish Dance, accessed September 22, 2025, [https://culkinschool.com/beginners-new-students/competitions/](https://culkinschool.com/beginners-new-students/competitions/)  
25. Rules \- The Nassau County Feis, accessed September 22, 2025, [https://www.nassauaohfeis.com/rules/](https://www.nassauaohfeis.com/rules/)  
26. The Basics \- Heart of Ireland, accessed September 22, 2025, [https://heartofirelandschool.com/faq-items/the-basics/](https://heartofirelandschool.com/faq-items/the-basics/)  
27. Dance Shoes – Teelin School of Irish Dance, accessed September 22, 2025, [https://teelin.com/TSID/teelin-gear/dance-shoes/](https://teelin.com/TSID/teelin-gear/dance-shoes/)  
28. First Feis Guide and Checklist \- Spokane Irish Dance\!, accessed September 22, 2025, [https://www.haranirishdance.com/first-feis-guide-and-checklist](https://www.haranirishdance.com/first-feis-guide-and-checklist)  
29. The Intricate World of Irish Dance Shoes: Soft Shoe vs. Hard Shoe \- iIrish newsmagazine, accessed September 22, 2025, [https://iirish.us/2024/06/01/the-intricate-world-of-irish-dance-shoes-soft-shoe-vs-hard-shoe/](https://iirish.us/2024/06/01/the-intricate-world-of-irish-dance-shoes-soft-shoe-vs-hard-shoe/)  
30. Light Shoe vs Heavy Shoe \- Oregon Irish Dance Academy, accessed September 22, 2025, [https://www.oregonirishdance.com/classtypes](https://www.oregonirishdance.com/classtypes)  
31. About Irish Dance & Music, accessed September 22, 2025, [https://www.rhodesirishdance.com/about-irish-dance-music](https://www.rhodesirishdance.com/about-irish-dance-music)  
32. Music Used for Irish Dance, accessed September 22, 2025, [https://www.cs.helsinki.fi/u/mkpalohe/](https://www.cs.helsinki.fi/u/mkpalohe/)  
33. Competing in Irish Dance \- Packing for a Feis | IrishCentral.com, accessed September 22, 2025, [https://www.irishcentral.com/culture/entertainment/competing-in-irish-dance-packing-for-a-feis-125807398-238094371](https://www.irishcentral.com/culture/entertainment/competing-in-irish-dance-packing-for-a-feis-125807398-238094371)  
34. First Feis Guide \- Deirdre O' Mara School of Irish Dance, accessed September 22, 2025, [https://deirdreomara.com/first-feis-guide/](https://deirdreomara.com/first-feis-guide/)  
35. Parent Primer | cida \- Celtic Irish Dance Academy, accessed September 22, 2025, [https://www.celticirishdanceacademy.com/parent-primer-feis-101](https://www.celticirishdanceacademy.com/parent-primer-feis-101)  
36. Irish dance \- Wikipedia, accessed September 22, 2025, [https://en.wikipedia.org/wiki/Irish\_dance](https://en.wikipedia.org/wiki/Irish_dance)  
37. How to end a feis dance? : r/irishdance \- Reddit, accessed September 22, 2025, [https://www.reddit.com/r/irishdance/comments/1l2skkp/how\_to\_end\_a\_feis\_dance/](https://www.reddit.com/r/irishdance/comments/1l2skkp/how_to_end_a_feis_dance/)  
38. Irish Dancing \- Studyclix, accessed September 22, 2025, [https://blob-static.studyclix.ie/static/content/file/attachments/3/3e5d7af5-3a4e-4f13-a3cb-61239e8061f7.pdf](https://blob-static.studyclix.ie/static/content/file/attachments/3/3e5d7af5-3a4e-4f13-a3cb-61239e8061f7.pdf)  
39. Everything You Need to Know About an Oireachtas \- Teelin Irish Dance, accessed September 22, 2025, [https://teelin.com/\_pdf/Oireachtas%20Info.pdf](https://teelin.com/_pdf/Oireachtas%20Info.pdf)  
40. Understanding Championship Scoring by Jim Montague The preliminary and championship events are the highest dance levels at, accessed September 22, 2025, [http://www.boyleschool.com/scoring.pdf](http://www.boyleschool.com/scoring.pdf)  
41. Feiseanna \- Kender Academy of Irish Dance, accessed September 22, 2025, [https://www.kenderacademy.com/feiseanna.html](https://www.kenderacademy.com/feiseanna.html)  
42. The World of Competitive Irish Dance \- 5 West magazine, accessed September 22, 2025, [https://www.5westmag.com/competitve-irish-dance/](https://www.5westmag.com/competitve-irish-dance/)  
43. Advice for Prelim? : r/irishdance \- Reddit, accessed September 22, 2025, [https://www.reddit.com/r/irishdance/comments/1bq4f02/advice\_for\_prelim/](https://www.reddit.com/r/irishdance/comments/1bq4f02/advice_for_prelim/)  
44. Full article: Dynamic flexible job shop scheduling problem considering multiple types of dynamic events \- Taylor & Francis Online, accessed September 22, 2025, [https://www.tandfonline.com/doi/full/10.1080/00207543.2025.2550454?src=](https://www.tandfonline.com/doi/full/10.1080/00207543.2025.2550454?src)  
45. Constraint-based Timetabling \- UniTime, accessed September 22, 2025, [https://www.unitime.org/papers/phdab05.pdf](https://www.unitime.org/papers/phdab05.pdf)  
46. Solving the timetabling problem using constraint satisfaction programming, accessed September 22, 2025, [https://ro.uow.edu.au/articles/thesis/Solving\_the\_timetabling\_problem\_using\_constraint\_satisfaction\_programming/27827784](https://ro.uow.edu.au/articles/thesis/Solving_the_timetabling_problem_using_constraint_satisfaction_programming/27827784)  
47. CONSTRAINT PROGRAMMING AND UNIVERSITY TIMETABLING \- The South African Journal of Industrial Engineering, accessed September 22, 2025, [https://sajie.journals.ac.za/pub/article/download/314/259/277](https://sajie.journals.ac.za/pub/article/download/314/259/277)  
48. Models and Algorithms for School Timetabling \- A Constraint-Programming Approach, accessed September 22, 2025, [https://edoc.ub.uni-muenchen.de/936/1/Marte\_Michael.pdf](https://edoc.ub.uni-muenchen.de/936/1/Marte_Michael.pdf)  
49. CSP: A Simulator For Multi-Agent Ranking Competitions \- arXiv, accessed September 22, 2025, [https://arxiv.org/pdf/2502.11197](https://arxiv.org/pdf/2502.11197)  
50. Feising FAQs | My Site \- Cara Rince, accessed September 22, 2025, [https://www.cararince.com/feising-faqs](https://www.cararince.com/feising-faqs)  
51. QuickFeis \- Apps on Google Play, accessed September 22, 2025, [https://play.google.com/store/apps/details?id=com.quickfeis](https://play.google.com/store/apps/details?id=com.quickfeis)  
52. QuickFeis on the App Store, accessed September 22, 2025, [https://apps.apple.com/us/app/quickfeis/id1535411032](https://apps.apple.com/us/app/quickfeis/id1535411032)  
53. 27 Key Performance Indicators (KPIs) for Events to Consider | Event Espresso, accessed September 22, 2025, [https://eventespresso.com/blog/kpis-for-events](https://eventespresso.com/blog/kpis-for-events)  
54. Event KPIs: 14 Must-Know Metrics I Sweap, accessed September 22, 2025, [https://www.sweap.io/en/blog/the-most-important-event-kpis](https://www.sweap.io/en/blog/the-most-important-event-kpis)  
55. Measuring Up: A Quick Guide to Important Meetings and Events KPIs, accessed September 22, 2025, [https://ideas.com/a-quick-guide-to-important-meetings-events-kpis/](https://ideas.com/a-quick-guide-to-important-meetings-events-kpis/)  
56. Here Are 19 Event KPIs You Need To Be Tracking \- EventsAir, accessed September 22, 2025, [https://www.eventsair.com/blog/event-kpis](https://www.eventsair.com/blog/event-kpis)  
57. Feisanna scheduling from year to year? : r/irishdance \- Reddit, accessed September 22, 2025, [https://www.reddit.com/r/irishdance/comments/1kftepk/feisanna\_scheduling\_from\_year\_to\_year/](https://www.reddit.com/r/irishdance/comments/1kftepk/feisanna_scheduling_from_year_to_year/)  
58. TIPS FOR COMPETITORS \- Teelin Irish Dance, accessed September 22, 2025, [https://teelin.com/\_pdf/Tips%20for%20Competitors.pdf](https://teelin.com/_pdf/Tips%20for%20Competitors.pdf)  
59. How are no-shows handled? \- Gen Con, accessed September 22, 2025, [https://www.gencon.com/forums/9-event-organizers-gms/topics/309-how-are-no-shows-handled](https://www.gencon.com/forums/9-event-organizers-gms/topics/309-how-are-no-shows-handled)  
60. Irish Dance First Feis: Tips for a Memorable Debut\!, accessed September 22, 2025, [https://www.cabeacademy.ie/blog/irish-dance-first-feis-tips-for-a-memorable-debut](https://www.cabeacademy.ie/blog/irish-dance-first-feis-tips-for-a-memorable-debut)  
61. Objective Functions for Minimizing Rescheduling Changes in ... \- MDPI, accessed September 22, 2025, [https://www.mdpi.com/2673-4052/6/3/30](https://www.mdpi.com/2673-4052/6/3/30)  
62. (PDF) Methods for Repair Based Scheduling \- ResearchGate, accessed September 22, 2025, [https://www.researchgate.net/publication/237133753\_Methods\_for\_Repair\_Based\_Scheduling](https://www.researchgate.net/publication/237133753_Methods_for_Repair_Based_Scheduling)  
63. What is Human-in-the-Loop (HITL) in AI & ML? \- Google Cloud, accessed September 22, 2025, [https://cloud.google.com/discover/human-in-the-loop](https://cloud.google.com/discover/human-in-the-loop)  
64. What Is Human In The Loop (HITL)? \- IBM, accessed September 22, 2025, [https://www.ibm.com/think/topics/human-in-the-loop](https://www.ibm.com/think/topics/human-in-the-loop)  
65. Iterative Repair Algorithm The following is the algorithm for the ..., accessed September 22, 2025, [https://www.cs.utexas.edu/\~pstone/Papers/DCAPS-IAC/DCAPS-IAC4.pdf](https://www.cs.utexas.edu/~pstone/Papers/DCAPS-IAC/DCAPS-IAC4.pdf)  
66. Human-in-the-loop \- Wikipedia, accessed September 22, 2025, [https://en.wikipedia.org/wiki/Human-in-the-loop](https://en.wikipedia.org/wiki/Human-in-the-loop)  
67. Introduction to Sports Scheduling: Leagues, Tournaments & Seasons \- YouTube, accessed September 22, 2025, [https://www.youtube.com/watch?v=\_E3mOpPilGM](https://www.youtube.com/watch?v=_E3mOpPilGM)  
68. Human-in-the-Loop AI (HITL) \- Complete Guide to Benefits, Best Practices & Trends for 2025, accessed September 22, 2025, [https://parseur.com/blog/human-in-the-loop-ai](https://parseur.com/blog/human-in-the-loop-ai)