// reserved keyword

import 'package:isar/isar.dart';

@collection
class Model {
  late int id;

  String? prop1;

  @Name('unique')
  String? prop2;
}
