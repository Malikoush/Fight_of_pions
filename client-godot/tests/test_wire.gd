extends TestCase
## Le fil : les nombres entiers redeviennent entiers, les valeurs étiquetées se lisent.


func test_whole_numbers_come_back_as_integers() -> bool:
	var message: Variant = Wire.decode('{"Place": {"row": 3.0, "col": 4}}')
	var place: Dictionary = message["Place"]
	equal(typeof(place["row"]), TYPE_INT, "rangée entière")
	equal(Wire.encode({"Place": place}), '{"Place":{"col":4,"row":3}}', "renvoyé tel quel")
	return true


func test_not_json_is_null() -> bool:
	equal(Wire.decode("bonjour"), null, "texte illisible")
	return true


func test_tagged_values_read_their_name_and_content() -> bool:
	equal(Wire.tag("EndTurn"), "EndTurn", "valeur sans donnée")
	equal(Wire.tag({"Place": {"row": 1}}), "Place", "valeur à donnée")
	equal(Wire.content({"Tokens": 2}), 2, "donnée")
	equal(Wire.tag({"a": 1, "b": 2}), "", "plusieurs clés : pas une valeur étiquetée")
	equal(Wire.content("Never"), null, "pas de donnée")
	return true
