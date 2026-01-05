import 'package:json_annotation/json_annotation.dart';

part 'model.g.dart';

@JsonSerializable()
class Model {
  final String id;
  final String name;
  final String provider;
  final String? description;
  final Map<String, dynamic>? parameters;

  Model({
    required this.id,
    required this.name,
    required this.provider,
    this.description,
    this.parameters,
  });

  factory Model.fromJson(Map<String, dynamic> json) => _$ModelFromJson(json);
  Map<String, dynamic> toJson() => _$ModelToJson(this);
}

