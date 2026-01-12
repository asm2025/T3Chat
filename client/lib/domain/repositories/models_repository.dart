import '../models/model.dart';

abstract class ModelsRepository {
  Future<List<Model>> listModels();
  Future<List<Model>> getAvailableModels();
}
