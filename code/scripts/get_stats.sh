leafs=$(jq '[ recurse(.children[]?) | select(.children == [] )] | length' < "$1")
nodes=$(jq '[ recurse(.children[]?) | .role ] | length' < "$1")
max_depth=$(jq 'def max_depth: if type == "object" or type == "array" then (map(. | max_depth) | max) + 1 else 0 end; max_depth' < "$1")
max_children=$(jq '[ recurse(.children[]?) | .children | length ] | max' < "$1")
roles=($(jq '[ recurse(.children[]?) | .role ] | unique | .[]' < "$1" | tr -d '\n' | tr '""' ' ' | tr -d '"'))
unique_roles=$(jq '[ recurse(.children[]?) | .role ] | unique | length' < "$1")
children_all=$(jq '[ recurse(.children[]?) | .children | length]' < "$1")
for i in $(seq 0 20); do
	two_or_less=$(echo "$children_all" | jq "[select(.[] <= $i)] | length")
	more_than_two=$(echo "$children_all" | jq "[select(.[] > $i)] | length")
	echo "> than $i: $more_than_two"
	echo "<= than $i: $two_or_less"
done
declare -a each_role
for role in "${roles[@]}"; do
	echo "$role"
	each_role+=($(jq "[ recurse(.children[]?) | select(.role == \"$role\") ] | length" < "$1"))
done

echo "Leafs: $leafs"
echo "Total nodes: $nodes"
echo "Unique Roles: $unique_roles"
for i in "${!each_role[@]}"; do
	echo -e "\t${roles[$i]}: ${each_role[$i]}"
done
echo "Max depth: $max_depth"
echo "Max children: $max_children"
