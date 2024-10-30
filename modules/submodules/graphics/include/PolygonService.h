#ifndef GOUDENGINE_POLYGONSERVICE_H
#define GOUDENGINE_POLYGONSERVICE_H

#include <vector>
#include <utility>

namespace GoudEngine {

class PolygonService {
public:
    virtual ~PolygonService() = default;

    virtual void DrawPolygon(const std::vector<std::pair<float, float>>& vertices) = 0;
};

} // namespace GoudEngine

#endif // GOUDENGINE_POLYGONSERVICE_H
