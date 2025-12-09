#!/bin/bash

# Script to quickly add many fake neighbor apartments for demo purposes

echo "🏢 Creating apartment building with 100 units for monitoring demo..."

# Common neighbor names for realistic demo
NAMES=(
    "smith" "johnson" "williams" "brown" "jones" "garcia" "miller" "davis" "rodriguez" "martinez"
    "hernandez" "lopez" "gonzales" "wilson" "anderson" "thomas" "taylor" "moore" "jackson" "martin"
    "lee" "perez" "thompson" "white" "harris" "sanchez" "clark" "ramirez" "lewis" "robinson"
    "walker" "young" "allen" "king" "wright" "scott" "torres" "nguyen" "hill" "flores"
    "green" "adams" "nelson" "baker" "hall" "rivera" "campbell" "mitchell" "carter" "roberts"
    "gomez" "phillips" "evans" "turner" "diaz" "parker" "cruz" "edwards" "collins" "reyes"
    "stewart" "morris" "morales" "murphy" "cook" "rogers" "gutierrez" "ortiz" "morgan" "cooper"
    "peterson" "bailey" "reed" "kelly" "howard" "ramos" "kim" "cox" "ward" "richardson"
    "watson" "brooks" "chavez" "wood" "james" "bennett" "gray" "mendoza" "ruiz" "hughes"
    "price" "alvarez" "castillo" "sanders" "patel" "myers" "long" "ross" "foster" "jimenez"
)

# Start from apartment 002 (001 is your real apartment)
for i in {2..100}; do
    APARTMENT_NUM=$(printf "%03d" $i)
    NAME_INDEX=$(( ($i - 2) % ${#NAMES[@]} ))
    NEIGHBOR_NAME="${NAMES[$NAME_INDEX]}"
    
    echo "Adding apartment $APARTMENT_NUM for $NEIGHBOR_NAME..."
    ./add-neighbor.sh $APARTMENT_NUM $NEIGHBOR_NAME &
    
    # Add some apartments in parallel, but not all at once to avoid overwhelming kubectl
    if (( $i % 10 == 0 )); then
        wait  # Wait for this batch to complete
        echo "✅ Completed batch up to apartment $APARTMENT_NUM"
        sleep 2
    fi
done

wait  # Wait for any remaining background processes

echo ""
echo "🎉 Demo apartment building created!"
echo "📊 Total apartments: 100 (1 real + 99 fake neighbors)"
echo "📈 View in Grafana: All apartments should appear in monitoring dashboard"
echo ""
echo "🔍 Check status:"
echo "   kubectl get namespaces | grep apartment"
echo "   kubectl get pods --all-namespaces | grep knx-homekit-bridge"
echo ""
echo "📋 Monitoring URLs:"
echo "   • Grafana: http://localhost:3000"
echo "   • Your real apartment: http://localhost:8080"