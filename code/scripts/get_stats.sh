leafs=$(jq '[ recurse(.children[]?) | select(.children == [] )] | length' < "$1")
nodes=$(jq '[ recurse(.children[]?) | .role ] | length' < "$1")
roles=($(jq '[ recurse(.children[]?) | .role ] | unique | .[]' < "$1" | tr -d '\n' | tr '""' ' ' | tr -d '"'))
unique_roles=$(jq '[ recurse(.children[]?) | .role ] | unique | length' < "$1")
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
